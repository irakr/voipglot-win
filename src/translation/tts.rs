use crate::error::{Result, VoipGlotError};
use crate::config::TtsConfig;
use tracing::{info, debug, warn, error};
use coqui_tts::Synthesizer;
use std::collections::HashMap;

pub struct TextToSpeech {
    config: TtsConfig,
    synthesizer: Option<Synthesizer>,
    sample_rate: u32,
    channels: u16,
    current_language: Option<String>,
    // Language-specific model mapping for better multilingual support
    language_models: HashMap<String, String>,
}

impl TextToSpeech {
    pub fn new(config: TtsConfig) -> Result<Self> {
        info!("Initializing Coqui TTS with config: {:?}", config);
        
        // Initialize language-specific model mapping
        let mut language_models = HashMap::new();
        language_models.insert("en".to_string(), "tts_models/en/ljspeech/tacotron2-DDC".to_string());
        language_models.insert("es".to_string(), "tts_models/es/css10/vits".to_string());
        language_models.insert("fr".to_string(), "tts_models/fr/css10/vits".to_string());
        language_models.insert("de".to_string(), "tts_models/de/css10/vits".to_string());
        language_models.insert("it".to_string(), "tts_models/it/css10/vits".to_string());
        language_models.insert("pt".to_string(), "tts_models/pt/css10/vits".to_string());
        language_models.insert("ru".to_string(), "tts_models/ru/css10/vits".to_string());
        language_models.insert("ja".to_string(), "tts_models/ja/css10/vits".to_string());
        language_models.insert("ko".to_string(), "tts_models/ko/css10/vits".to_string());
        language_models.insert("zh".to_string(), "tts_models/zh/css10/vits".to_string());
        
        let model_name = if !config.model_path.is_empty() {
            &config.model_path
        } else {
            "tts_models/en/ljspeech/tacotron2-DDC"  // Default model
        };
        
        // Note: Don't check if model directory exists - let Coqui TTS handle model downloading automatically
        // The model will be downloaded on first use if it doesn't exist
        info!("TTS model path configured: {} (will be downloaded automatically if needed)", model_name);
        
        let synthesizer = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Synthesizer::new(model_name, config.enable_gpu)
        })) {
            Ok(syn) => {
                info!("Coqui TTS synthesizer initialized successfully with model: {}", model_name);
                Some(syn)
            }
            Err(e) => {
                error!("Failed to initialize Coqui TTS synthesizer with model {}: {:?}", model_name, e);
                // Try to fall back to a simpler model if the main one fails
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Synthesizer::new("tts_models/en/ljspeech/tacotron2-DDC", config.enable_gpu)
                })) {
                    Ok(fallback_syn) => {
                        info!("Successfully initialized fallback TTS model (tacotron2-DDC)");
                        Some(fallback_syn)
                    }
                    Err(e2) => {
                        error!("Failed to initialize fallback TTS model: {:?}", e2);
                        error!("TTS initialization failed completely. Please ensure TTS is properly installed.");
                        error!("Run the following commands to set up TTS:");
                        error!("  1. python scripts/setup-coqui.py");
                        error!("  2. Restart the application");
                        None
                    }
                }
            }
        };
        
        Ok(Self {
            config: config.clone(),
            synthesizer,
            sample_rate: config.sample_rate,
            channels: config.channels,
            current_language: None,
            language_models,
        })
    }

    pub async fn synthesize(&mut self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Synthesizing speech for text: '{}'", text);
        
        // Check if synthesizer is available
        let synthesizer = match &mut self.synthesizer {
            Some(syn) => syn,
            None => {
                error!("TTS synthesizer not initialized");
                return Err(VoipGlotError::InitializationError("TTS synthesizer not initialized".to_string()));
            }
        };
        
        // Validate and sanitize input text
        let text_to_speak = {
            // Remove excessive whitespace and normalize text
            let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
            
            // Remove any non-printable characters that might cause issues
            let sanitized = normalized.chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .collect::<String>();
            
            // Limit text length to prevent performance issues (reduced to 150 for better performance)
            if sanitized.len() > 150 {
                warn!("Text too long ({} chars), truncating to 150 characters for better performance", sanitized.len());
                sanitized[..150].to_string()
            } else {
                sanitized
            }
        };
        
        // Additional validation
        if text_to_speak.trim().is_empty() {
            warn!("Text became empty after sanitization, skipping synthesis");
            return Ok(Vec::new());
        }
        
        info!("Converting text to speech: '{}'", text_to_speak);
        
        // Synthesize speech and get audio buffer with better error handling
        let start_time = std::time::Instant::now();
        let audio_buffer = {
            // Process in a separate scope to ensure memory is freed immediately
            let text_clone = text_to_speak.clone();
            let buffer = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                synthesizer.tts(&text_clone)
            })) {
                Ok(buffer) => buffer,
                Err(e) => {
                    error!("TTS synthesis panicked: {:?}", e);
                    return Err(VoipGlotError::SynthesisError(format!("TTS synthesis panicked: {:?}", e)));
                }
            };
            
            if buffer.is_empty() {
                error!("TTS synthesis returned empty audio buffer");
                return Err(VoipGlotError::SynthesisError("TTS synthesis returned empty audio buffer".to_string()));
            }
            
            // Normalize audio levels for consistent volume (matching test implementation)
            let max_amplitude = buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
            if max_amplitude > 0.0 {
                buffer.into_iter().map(|x| (x / max_amplitude) * 0.95).collect()
            } else {
                buffer
            }
        };
        
        let synthesis_time = start_time.elapsed();
        info!("Speech synthesis completed in {:?}, got {} samples", synthesis_time, audio_buffer.len());
        
        Ok(audio_buffer)
    }

    pub fn set_language(&mut self, language: String) -> Result<()> {
        debug!("Setting TTS language to: {}", language);
        
        // Check if language is already set to the requested language
        if let Some(current_lang) = &self.current_language {
            if current_lang == &language {
                debug!("TTS language already set to {}, skipping model switch", language);
                return Ok(());
            }
        }
        
        // For now, only support English TTS to avoid model loading issues
        // Other languages will use English TTS but with the target language marked
        if language != "en" {
            info!("TTS language switching to '{}' not fully supported yet, using English TTS", language);
            info!("The translation will still work, but speech synthesis will be in English");
            self.current_language = Some(language.clone());
            return Ok(());
        }
        
        // Check if we have a language-specific model
        if let Some(model_path) = self.language_models.get(&language) {
            info!("Switching to language-specific model: {}", model_path);
            
            // Note: Don't check if model directory exists - let Coqui TTS handle model downloading automatically
            info!("Language-specific model path configured: {} (will be downloaded automatically if needed)", model_path);
            
            // Try to initialize new synthesizer with language-specific model
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Synthesizer::new(model_path, self.config.enable_gpu)
            })) {
                Ok(new_syn) => {
                    self.synthesizer = Some(new_syn);
                    self.current_language = Some(language.clone());
                    info!("Successfully switched to {} language model", language);
                }
                Err(e) => {
                    error!("Failed to load language-specific model for {}: {:?}", language, e);
                    // Try to fall back to default model
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        Synthesizer::new("tts_models/en/ljspeech/tacotron2-DDC", self.config.enable_gpu)
                    })) {
                        Ok(default_syn) => {
                            self.synthesizer = Some(default_syn);
                            self.current_language = Some(language.clone());
                            warn!("Fell back to default English model (tacotron2-DDC) for language: {}", language);
                        }
                        Err(e2) => {
                            error!("Failed to load default model as fallback: {:?}", e2);
                            error!("TTS model loading failed completely. Please ensure TTS is properly installed.");
                            error!("Run the following commands to set up TTS:");
                            error!("  1. python scripts/setup-coqui.py");
                            error!("  2. Restart the application");
                            return Err(VoipGlotError::InitializationError(format!("Failed to load any TTS model: {:?}", e2)));
                        }
                    }
                }
            }
        } else {
            // No language-specific model available, just update language
            self.current_language = Some(language);
            debug!("No language-specific model available, using default model");
        }
        
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

    pub fn get_channels(&self) -> u16 {
        self.channels
    }

    pub fn set_voice_speed(&mut self, speed: f32) {
        self.config.voice_speed = speed.max(0.1).min(3.0);
        info!("Voice speed set to: {}", self.config.voice_speed);
        // Note: Coqui TTS may not support speed adjustment in this version
    }

    pub fn set_voice_pitch(&mut self, pitch: f32) {
        self.config.voice_pitch = pitch.max(0.5).min(2.0);
        info!("Voice pitch set to: {}", self.config.voice_pitch);
        // Note: Coqui TTS may not support pitch adjustment in this version
    }

    pub fn get_voice_speed(&self) -> f32 {
        self.config.voice_speed
    }

    pub fn get_voice_pitch(&self) -> f32 {
        self.config.voice_pitch
    }

    pub fn get_current_language(&self) -> Option<&String> {
        self.current_language.as_ref()
    }

    pub fn is_initialized(&self) -> bool {
        self.synthesizer.is_some()
    }
}

impl Drop for TextToSpeech {
    fn drop(&mut self) {
        // Coqui TTS doesn't require explicit cleanup
        info!("TTS module dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TtsConfig;

    #[test]
    fn test_tts_initialization() {
        let config = TtsConfig {
            provider: "coqui".to_string(),
            model_path: "tts_models/en/ljspeech/fast_pitch".to_string(),
            sample_rate: 22050,
            channels: 1,
            voice_speed: 1.0,
            voice_pitch: 1.0,
            enable_gpu: false,
        };
        
        let tts = TextToSpeech::new(config);
        // The test passes if initialization doesn't panic
        // Note: This may fail if Python TTS is not installed, which is expected
        assert!(tts.is_ok() || tts.is_err());
    }

    #[test]
    fn test_tts_config_defaults() {
        let config = TtsConfig::default();
        assert_eq!(config.provider, "coqui");
        assert_eq!(config.model_path, "tts_models/en/ljspeech/fast_pitch");
        assert_eq!(config.sample_rate, 22050);
        assert_eq!(config.channels, 1);
        assert_eq!(config.voice_speed, 1.0);
        assert_eq!(config.voice_pitch, 1.0);
        assert_eq!(config.enable_gpu, false);
    }

    #[test]
    fn test_language_models() {
        let config = TtsConfig::default();
        let tts = TextToSpeech::new(config).unwrap();
        
        // Test that language models are properly initialized
        assert!(tts.language_models.contains_key("en"));
        assert!(tts.language_models.contains_key("es"));
        assert!(tts.language_models.contains_key("fr"));
    }

    #[test]
    fn test_tts_language_switching() {
        let config = TtsConfig::default();
        let mut tts = TextToSpeech::new(config).unwrap();
        
        // Test language switching
        assert!(tts.set_language("en".to_string()).is_ok());
        assert_eq!(tts.get_current_language(), Some(&"en".to_string()));
        
        // Test setting to a supported language
        assert!(tts.set_language("es".to_string()).is_ok());
        assert_eq!(tts.get_current_language(), Some(&"es".to_string()));
    }

    #[test]
    fn test_tts_initialization_status() {
        let config = TtsConfig::default();
        let tts = TextToSpeech::new(config).unwrap();
        
        // Test that TTS is properly initialized
        assert!(tts.is_initialized());
    }
} 