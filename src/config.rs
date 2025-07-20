use serde::{Deserialize, Serialize, Deserializer};

use crate::error::{Result, VoipGlotError};

// Custom deserialization function to convert empty strings to None
fn deserialize_optional_string<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.filter(|s| !s.trim().is_empty()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio_input: AudioInputConfig,
    pub audio_output: AudioOutputConfig,
    pub stt: SttConfig,
    pub translation: TranslationConfig,
    pub tts: TtsConfig,
    pub processing: ProcessingConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInputConfig {
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub input_device: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub latency_ms: u32,
    pub vb_cable_device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutputConfig {
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub output_device: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub provider: String,
    pub model_path: String,
    pub sample_rate: f32,
    pub enable_partial_results: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub provider: String,
    pub model_path: String,
    pub source_language: String,
    pub target_language: String,
    pub num_threads: usize,
    pub device: String,
    pub max_batch_size: usize,
    pub beam_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub provider: String,
    pub model_path: String,
    pub voice_speed: f32,
    pub voice_pitch: f32,
    pub enable_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub chunk_duration_ms: u32,
    pub silence_threshold: f32,
    pub noise_reduction: bool,
    pub echo_cancellation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub log_file: Option<String>,
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
            audio_input: AudioInputConfig::default(),
            audio_output: AudioOutputConfig::default(),
            stt: SttConfig::default(),
            translation: TranslationConfig::default(),
            tts: TtsConfig::default(),
            processing: ProcessingConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for AudioInputConfig {
    fn default() -> Self {
        Self {
            input_device: None,
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
            latency_ms: 50,
            vb_cable_device: "CABLE Input (VB-Audio Virtual Cable)".to_string(),
        }
    }
}

impl Default for AudioOutputConfig {
    fn default() -> Self {
        Self {
            output_device: None,
            sample_rate: 48000,
            channels: 2,
            buffer_size: 2048,
            latency_ms: 100,
        }
    }
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            provider: "vosk".to_string(),
            model_path: "models/vosk-model-small-en-us-0.15".to_string(),
            sample_rate: 16000.0,
            enable_partial_results: true,
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            provider: "ct2".to_string(),
            model_path: "models/nllb-200-ct2".to_string(),
            source_language: "en".to_string(),
            target_language: "es".to_string(),
            num_threads: 4,
            device: "cpu".to_string(),
            max_batch_size: 32,
            beam_size: 4,
        }
    }
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: "coqui".to_string(),
            model_path: "tts_models/en/ljspeech/tacotron2-DDC".to_string(),
            voice_speed: 1.0,
            voice_pitch: 1.0,
            enable_gpu: false,
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

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "simple".to_string(),
            log_file: None,
        }
    }
} 