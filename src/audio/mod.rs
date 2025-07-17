pub mod capture;
pub mod playback;
pub mod processing;

use crate::config::{AudioConfig, ProcessingConfig};
use crate::error::Result;
use crate::translation::Translator;
use tracing::{info, error, debug};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use capture::AudioCapture;
pub use playback::AudioPlayback;
pub use processing::AudioProcessor;

pub struct AudioManager {
    config: AudioConfig,
    capture: Option<AudioCapture>,
    playback: Option<AudioPlayback>,
    processor: AudioProcessor,
    shutdown_signal: Arc<AtomicBool>,
}

impl AudioManager {
    pub fn new(audio_config: AudioConfig, processing_config: ProcessingConfig) -> Result<Self> {
        info!("Initializing AudioManager with audio config: {:?}", audio_config);
        info!("Processing config: {:?}", processing_config);
        
        let mut processor = AudioProcessor::new(audio_config.clone())?;
        processor.update_processing_config(
            processing_config.silence_threshold,
            processing_config.chunk_duration_ms
        );
        
        Ok(Self {
            config: audio_config,
            capture: None,
            playback: None,
            processor,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn set_shutdown_signal(&mut self, shutdown_signal: Arc<AtomicBool>) {
        self.shutdown_signal = shutdown_signal;
    }

    pub async fn start_processing(&mut self, mut translator: Translator) -> Result<()> {
        info!("Starting audio processing pipeline");
        
        // Initialize audio capture
        self.capture = Some(AudioCapture::new(self.config.clone())?);
        info!("Audio capture initialized");
        
        // Initialize audio playback
        self.playback = Some(AudioPlayback::new(self.config.clone())?);
        info!("Audio playback initialized");
        
        // Start the processing loop
        self.run_processing_loop(&mut translator).await?;
        
        Ok(())
    }

    async fn run_processing_loop(&mut self, translator: &mut Translator) -> Result<()> {
        info!("Starting audio processing loop");
        
        let capture = self.capture.as_mut()
            .ok_or_else(|| crate::error::VoipGlotError::Audio("Capture not initialized".to_string()))?;
        
        let playback = self.playback.as_mut()
            .ok_or_else(|| crate::error::VoipGlotError::Audio("Playback not initialized".to_string()))?;
        
        // Start capture and playback streams
        capture.start()?;
        playback.start()?;
        
        info!("Audio streams started successfully");
        
        // Main processing loop
        loop {
            // Check for shutdown signal
            if self.shutdown_signal.load(Ordering::Relaxed) {
                info!("Shutdown signal received, stopping audio processing");
                break;
            }
            
            match capture.read_audio_chunk().await {
                Ok(audio_data) => {
                    debug!("Received audio chunk of {} samples", audio_data.len());
                    
                    // Process the audio through the translation pipeline
                    match self.processor.process_audio(audio_data, translator).await {
                        Ok(translated_audio) => {
                            if let Some(translated_audio) = translated_audio {
                                debug!("Sending translated audio to playback");
                                if let Err(e) = playback.write_audio_chunk(translated_audio).await {
                                    error!("Failed to write audio chunk: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error processing audio: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading audio chunk: {}", e);
                    // Add a small delay to prevent tight loop on errors
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
            
            // Add a small delay to prevent excessive CPU usage
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
        
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping audio processing");
        
        if let Some(capture) = &mut self.capture {
            capture.stop()?;
        }
        
        if let Some(playback) = &mut self.playback {
            playback.stop()?;
        }
        
        info!("Audio processing stopped");
        Ok(())
    }
} 