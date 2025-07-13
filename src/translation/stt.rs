use crate::error::{Result, VoipGlotError};
use crate::config::SttConfig;
use tracing::{info, error, debug, warn};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer, DecodingState};

pub struct SpeechToText {
    config: SttConfig,
    model: Option<Model>,
    recognizer: Option<Arc<Mutex<Recognizer>>>,
    sample_rate: u32,
}

impl SpeechToText {
    pub fn new(config: SttConfig) -> Result<Self> {
        info!("Initializing VOSK Speech-to-Text with config: {:?}", config);
        
        // Validate model path
        if !std::path::Path::new(&config.model_path).exists() {
            return Err(VoipGlotError::Configuration(
                format!("VOSK model path does not exist: {}", config.model_path)
            ));
        }
        
        info!("Loading VOSK model from: {}", config.model_path);
        
        // Load VOSK model
        let model = Model::new(&config.model_path)
            .ok_or_else(|| {
                error!("Failed to load VOSK model from: {}", config.model_path);
                VoipGlotError::Configuration(
                    format!("Failed to load VOSK model from: {}", config.model_path)
                )
            })?;
        
        info!("VOSK model loaded successfully");
        
        // Create recognizer with the model's sample rate
        let sample_rate = config.sample_rate as u32;
        let recognizer = Recognizer::new(&model, sample_rate as f32)
            .ok_or_else(|| {
                error!("Failed to create VOSK recognizer");
                VoipGlotError::Configuration("Failed to create VOSK recognizer".to_string())
            })?;
        
        info!("VOSK recognizer created with sample rate: {}Hz", sample_rate);
        
        Ok(Self {
            config,
            model: Some(model),
            recognizer: Some(Arc::new(Mutex::new(recognizer))),
            sample_rate,
        })
    }

    pub async fn transcribe(&self, audio_data: Vec<f32>) -> Result<String> {
        debug!("Transcribing audio with {} samples", audio_data.len());
        
        if audio_data.is_empty() {
            return Ok(String::new());
        }
        
        let recognizer = self.recognizer.as_ref()
            .ok_or_else(|| VoipGlotError::Audio("Recognizer not initialized".to_string()))?;
        
        // Convert f32 samples to i16 for VOSK
        let samples: Vec<i16> = audio_data
            .iter()
            .map(|&sample| {
                // Ensure the sample is in the [-1.0, 1.0] range before converting
                let clamped = sample.max(-1.0).min(1.0);
                (clamped * 32767.0) as i16
            })
            .collect();
        
        // Process audio with VOSK
        if let Ok(mut rec) = recognizer.lock() {
            let state = rec.accept_waveform(&samples);
            
            match state {
                DecodingState::Finalized => {
                    let result = rec.result();
                    debug!("Got final result from VOSK: {:?}", result);
                    
                    if let Some(text) = self.extract_text_from_result(&format!("{:?}", result)) {
                        if !text.is_empty() {
                            info!("Transcribed text: '{}'", text);
                            return Ok(text);
                        }
                    }
                },
                DecodingState::Running => {
                    if self.config.enable_partial_results {
                        let partial = rec.partial_result();
                        debug!("Got partial result: {:?}", partial);
                        
                        if let Some(text) = self.extract_text_from_result(&format!("{:?}", partial)) {
                            if !text.is_empty() {
                                debug!("Partial transcription: '{}'", text);
                                return Ok(text);
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        Ok(String::new())
    }

    fn extract_text_from_result(&self, result: &str) -> Option<String> {
        debug!("Processing VOSK result: {}", result);
        
        // For CompleteResultSingle format
        if result.contains("CompleteResultSingle") {
            if let Some(text_start) = result.find("text: \"") {
                if let Some(text_end) = result[text_start..].find("\"}") {
                    let text = result[text_start + 7..text_start + text_end].to_string();
                    debug!("Extracted text from CompleteResultSingle: {}", text);
                    return Some(text);
                }
            }
        }
        
        // For other result formats, try to extract any text field
        if let Some(text_start) = result.find("\"text\":\"") {
            if let Some(text_end) = result[text_start + 8..].find('\"') {
                let text = result[text_start + 8..text_start + 8 + text_end].to_string();
                debug!("Extracted text from JSON format: {}", text);
                return Some(text);
            }
        }
        
        debug!("No text could be extracted from result");
        None
    }

    pub fn set_language(&mut self, _language: String) -> Result<()> {
        // VOSK models are language-specific, so changing language would require
        // loading a different model. For now, we'll just log this.
        warn!("Language change requested but VOSK model is language-specific. Current model: {}", self.config.model_path);
        Ok(())
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        vec![
            "en".to_string(), // English
            "es".to_string(), // Spanish
            "fr".to_string(), // French
            "de".to_string(), // German
            "it".to_string(), // Italian
            "pt".to_string(), // Portuguese
            "ru".to_string(), // Russian
            "ja".to_string(), // Japanese
            "ko".to_string(), // Korean
            "zh".to_string(), // Chinese
        ]
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn reset(&mut self) -> Result<()> {
        debug!("Resetting VOSK recognizer");
        
        if let Some(model) = &self.model {
            let recognizer = Recognizer::new(model, self.sample_rate as f32)
                .ok_or_else(|| VoipGlotError::Audio("Failed to recreate VOSK recognizer".to_string()))?;
            
            self.recognizer = Some(Arc::new(Mutex::new(recognizer)));
            info!("VOSK recognizer reset successfully");
        }
        
        Ok(())
    }
} 