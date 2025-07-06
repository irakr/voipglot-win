use serde::{Deserialize, Serialize};
use crate::error::{Result, VoipGlotError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub translation: TranslationConfig,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationProvider {
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub chunk_duration_ms: u32,
    pub silence_threshold: f32,
    pub noise_reduction: bool,
    pub echo_cancellation: bool,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::Environment::with_prefix("VOIPGLOT"))
            .build()
            .map_err(|e| VoipGlotError::Configuration(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| VoipGlotError::Configuration(e.to_string()))
    }

    pub fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            translation: TranslationConfig::default(),
            processing: ProcessingConfig::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device: None,
            output_device: None,
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
            latency_ms: 50,
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            source_language: "en".to_string(),
            target_language: "es".to_string(),
        }
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            chunk_duration_ms: 1000,
            silence_threshold: 0.01,
            noise_reduction: true,
            echo_cancellation: true,
        }
    }
} 