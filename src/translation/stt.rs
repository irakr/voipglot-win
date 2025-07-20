use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use dasp::Sample;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info};
use vosk::{DecodingState, Model, Recognizer};

use crate::config::AppConfig;

pub struct STTProcessor {
    model: Model,
    audio_stream: Option<cpal::Stream>,
    text_tx: mpsc::UnboundedSender<String>,
    config: AppConfig,
}

impl STTProcessor {
    pub fn new(config: AppConfig, text_tx: mpsc::UnboundedSender<String>) -> Result<Self> {
        info!("Initializing VOSK STT processor");
        
        // Load VOSK model
        let model_path = &config.stt.model_path;
        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow::anyhow!("VOSK model not found at: {}", model_path));
        }
        
        let model = Model::new(model_path)
            .ok_or_else(|| anyhow::anyhow!("Failed to load VOSK model from: {}", model_path))?;
        
        info!("VOSK model loaded successfully from: {}", model_path);
        
        // We'll create the recognizer with the correct sample rate when we get the audio device config
        
        Ok(Self {
            model,
            audio_stream: None,
            text_tx,
            config,
        })
    }
    
    pub fn start_audio_capture(&mut self) -> Result<()> {
        info!("Starting audio capture for STT");
        
        // Get default input device
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
        
        info!("Using input device: {}", device.name().unwrap_or_default());
        
        // Get device config
        let config = device
            .default_input_config()
            .map_err(|e| anyhow::anyhow!("Failed to get input config: {}", e))?;
        
        let channels = config.channels();
        let sample_rate = config.sample_rate().0;
        
        info!("Audio config: {}Hz, {} channels, {:?} format", 
              sample_rate, channels, config.sample_format());
        
        // Create recognizer with the actual audio device sample rate (like the test app)
        let device_sample_rate = config.sample_rate().0 as f32;
        let mut recognizer = Recognizer::new(&self.model, device_sample_rate)
            .ok_or_else(|| anyhow::anyhow!("Failed to create VOSK recognizer"))?;
        
        // Configure recognizer like the working test app
        recognizer.set_max_alternatives(10);
        recognizer.set_words(true);
        recognizer.set_partial_words(true);
        
        let recognizer = Arc::new(Mutex::new(recognizer));
        
        let text_tx = self.text_tx.clone();
        
        // Build audio stream like the working test app
        let err_fn = |err| {
            error!("Audio stream error: {}", err);
        };
        
        let recognizer_clone = recognizer.clone();
        let text_tx_clone = text_tx.clone();
        let stream = match config.sample_format() {
            SampleFormat::I8 => device.build_input_stream(
                &config.into(),
                move |data: &[i8], _| {
                    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
                    let data = if channels != 1 {
                        Self::stereo_to_mono(&data)
                    } else {
                        data
                    };
                    
                    Self::recognize(&mut recognizer_clone.lock().unwrap(), &data, &text_tx_clone);
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let data = if channels != 1 {
                        Self::stereo_to_mono(data)
                    } else {
                        data.to_vec()
                    };
                    
                    Self::recognize(&mut recognizer_clone.lock().unwrap(), &data, &text_tx_clone);
                },
                err_fn,
                None,
            ),
            SampleFormat::I32 => device.build_input_stream(
                &config.into(),
                move |data: &[i32], _| {
                    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
                    let data = if channels != 1 {
                        Self::stereo_to_mono(&data)
                    } else {
                        data
                    };
                    
                    Self::recognize(&mut recognizer_clone.lock().unwrap(), &data, &text_tx_clone);
                },
                err_fn,
                None,
            ),
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
                    let data = if channels != 1 {
                        Self::stereo_to_mono(&data)
                    } else {
                        data
                    };
                    
                    Self::recognize(&mut recognizer_clone.lock().unwrap(), &data, &text_tx_clone);
                },
                err_fn,
                None,
            ),
            sample_format => {
                return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format));
            }
        }
        .map_err(|e| anyhow::anyhow!("Failed to build input stream: {}", e))?;
        
        // Start the stream
        stream.play().map_err(|e| anyhow::anyhow!("Failed to play stream: {}", e))?;
        
        // Store the stream to keep it alive
        self.audio_stream = Some(stream);
        
        info!("Audio capture started successfully");
        Ok(())
    }
    
    fn recognize(
        recognizer: &mut Recognizer,
        data: &[i16],
        text_tx: &mpsc::UnboundedSender<String>,
    ) {
        let state = recognizer.accept_waveform(data);
        
        match state {
            DecodingState::Running => {
                // Skip partial results - don't print them
            }
            DecodingState::Finalized => {
                // Result will always be multiple because we called set_max_alternatives
                if let Some(results) = recognizer.result().multiple() {
                    if let Some(best) = results.alternatives.first() {
                        if !best.text.is_empty() {
                            info!("STT: \"{}\" (confidence: {:.2})", 
                                  best.text, best.confidence);
                            
                            // Send transcribed text to the next module
                            if let Err(e) = text_tx.send(best.text.to_string()) {
                                error!("Failed to send transcribed text: {}", e);
                            }
                        }
                    }
                }
            }
            DecodingState::Failed => {
                error!("VOSK recognition failed");
            }
        }
    }
    
    fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
        let mut result = Vec::with_capacity(input_data.len() / 2);
        result.extend(
            input_data
                .chunks_exact(2)
                .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
        );
        result
    }
    
    pub fn stop(&mut self) {
        info!("Stopping STT processor");
        // Drop the stream to stop audio capture
        self.audio_stream = None;
    }
    
    pub async fn start_processing(&mut self) -> Result<()> {
        info!("Starting STT processing");
        
        // Start audio capture
        self.start_audio_capture()?;
        
        // Keep the task alive - audio processing happens in the background stream
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
