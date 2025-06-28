use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{Result, VoipGlotError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub translation: TranslationConfig,
    pub api: ApiConfig,
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
    pub stt_provider: SttProvider,
    pub translation_provider: TranslationProvider,
    pub tts_provider: TtsProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SttProvider {
    Whisper,
    Azure,
    Google,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationProvider {
    DeepL,
    Google,
    Azure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TtsProvider {
    Azure,
    ElevenLabs,
    Google,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub deepl_api_key: Option<String>,
    pub azure_speech_key: Option<String>,
    pub azure_region: Option<String>,
    pub elevenlabs_api_key: Option<String>,
    pub google_api_key: Option<String>,
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
            api: ApiConfig::default(),
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
            stt_provider: SttProvider::Whisper,
            translation_provider: TranslationProvider::DeepL,
            tts_provider: TtsProvider::Azure,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            deepl_api_key: None,
            azure_speech_key: None,
            azure_region: None,
            elevenlabs_api_key: None,
            google_api_key: None,
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