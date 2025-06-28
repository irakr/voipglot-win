pub mod capture;
pub mod playback;
pub mod processing;

use crate::config::AudioConfig;
use crate::error::Result;
use crate::translation::Translator;
use tracing::{info, error, debug};

pub use capture::AudioCapture;
pub use playback::AudioPlayback;
pub use processing::AudioProcessor;

pub struct AudioManager {
    config: AudioConfig,
    capture: Option<AudioCapture>,
    playback: Option<AudioPlayback>,
    processor: AudioProcessor,
}

impl AudioManager {
    pub fn new(config: AudioConfig) -> Result<Self> {
        info!("Initializing AudioManager with config: {:?}", config);
        
        let processor = AudioProcessor::new(config.clone())?;
        
        Ok(Self {
            config,
            capture: None,
            playback: None,
            processor,
        })
    }

    pub async fn start_processing(&mut self, translator: Translator) -> Result<()> {
        info!("Starting audio processing pipeline");
        
        // Initialize audio capture
        self.capture = Some(AudioCapture::new(self.config.clone())?);
        info!("Audio capture initialized");
        
        // Initialize audio playback
        self.playback = Some(AudioPlayback::new(self.config.clone())?);
        info!("Audio playback initialized");
        
        // Start the processing loop
        self.run_processing_loop(translator).await?;
        
        Ok(())
    }

    async fn run_processing_loop(&mut self, translator: Translator) -> Result<()> {
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
            match capture.read_audio_chunk().await {
                Ok(audio_data) => {
                    debug!("Received audio chunk of {} samples", audio_data.len());
                    
                    // Process the audio through the translation pipeline
                    match self.processor.process_audio(audio_data, &translator).await {
                        Ok(translated_audio) => {
                            if let Some(translated_audio) = translated_audio {
                                debug!("Sending translated audio to playback");
                                playback.write_audio_chunk(translated_audio).await?;
                            }
                        }
                        Err(e) => {
                            error!("Error processing audio: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading audio chunk: {}", e);
                    break;
                }
            }
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