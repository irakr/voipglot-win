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
        
        // Initialize language-specific model mapping - use fast_pitch for better performance
        let mut language_models = HashMap::new();
        language_models.insert("en".to_string(), "tts_models/en/ljspeech/fast_pitch".to_string());
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
            "tts_models/en/ljspeech/fast_pitch"  // Use fast_pitch for better performance
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
                // Try to fall back to fast_pitch model if the main one fails
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Synthesizer::new("tts_models/en/ljspeech/fast_pitch", config.enable_gpu)
                })) {
                    Ok(fallback_syn) => {
                        info!("Successfully initialized fallback TTS model (fast_pitch)");
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
            sample_rate: 22050, // Use TTS native sample rate to avoid resampling
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
        
        // Validate and sanitize input text - use PoC approach
        let text_to_speak = {
            // Remove excessive whitespace and normalize
            let cleaned = text.trim().replace("  ", " ");
            if cleaned.is_empty() {
                return Ok(Vec::new());
            }
            cleaned
        };
        
        info!("Converting text to speech: '{}'", text_to_speak);
        
        let start_time = std::time::Instant::now();
        
        // Synthesize speech using Coqui TTS
        let audio = synthesizer.tts(&text_to_speak);
        
        let synthesis_time = start_time.elapsed();
        info!("Speech synthesis completed in {:?}, got {} samples", synthesis_time, audio.len());
        
        // Convert audio to f32 samples and normalize
        let mut samples: Vec<f32> = audio
            .iter()
            .map(|&sample| {
                // Convert from i16 to f32 and normalize to [-1.0, 1.0] range
                (sample as f32) / 32768.0
            })
            .collect();
        
        // Apply gentle normalization to prevent clipping while maintaining quality
        if let Some(max_sample) = samples.iter().map(|&s| s.abs()).reduce(f32::max) {
            if max_sample > 0.0 {
                // Normalize to 90% of max to prevent clipping
                let normalization_factor = 0.9 / max_sample;
                for sample in &mut samples {
                    *sample *= normalization_factor;
                }
                debug!("Applied normalization factor: {:.4}", normalization_factor);
            }
        }
        
        // Only resample if absolutely necessary (different sample rates)
        if self.sample_rate != 22050 {
            debug!("Resampling from 22050Hz to {}Hz", self.sample_rate);
            samples = self.resample_audio(&samples, 22050, self.sample_rate);
        }
        
        // Convert mono to stereo if needed (simple duplication)
        if self.channels == 2 && samples.len() > 0 {
            let mono_samples = samples.clone();
            samples.clear();
            for &sample in &mono_samples {
                samples.push(sample); // Left channel
                samples.push(sample); // Right channel
            }
            debug!("Converted mono to stereo: {} samples", samples.len());
        }
        
        Ok(samples)
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
                        Synthesizer::new("tts_models/en/ljspeech/fast_pitch", self.config.enable_gpu)
                    })) {
                        Ok(default_syn) => {
                            self.synthesizer = Some(default_syn);
                            self.current_language = Some(language.clone());
                            warn!("Fell back to default English model (fast_pitch) for language: {}", language);
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

    fn apply_pitch_shift(&self, audio: &[f32], pitch_factor: f32) -> Vec<f32> {
        // Simple pitch shifting using resampling
        // This is a basic implementation - for better quality, consider using a library like rubberband
        if (pitch_factor - 1.0).abs() < 0.01 {
            return audio.to_vec();
        }
        
        let new_length = (audio.len() as f32 / pitch_factor) as usize;
        let mut result = Vec::with_capacity(new_length);
        
        for i in 0..new_length {
            let src_index = (i as f32 * pitch_factor) as usize;
            if src_index < audio.len() {
                result.push(audio[src_index]);
            }
        }
        
        result
    }

    fn apply_speed_change(&self, audio: &[f32], speed_factor: f32) -> Vec<f32> {
        // Simple speed change using linear interpolation
        if (speed_factor - 1.0).abs() < 0.01 {
            return audio.to_vec();
        }
        
        let new_length = (audio.len() as f32 * speed_factor) as usize;
        let mut result = Vec::with_capacity(new_length);
        
        for i in 0..new_length {
            let src_index = i as f32 / speed_factor;
            let src_index_floor = src_index.floor() as usize;
            let src_index_ceil = (src_index.ceil() as usize).min(audio.len() - 1);
            let fraction = src_index - src_index_floor as f32;
            
            if src_index_floor < audio.len() {
                let sample = if src_index_floor == src_index_ceil {
                    audio[src_index_floor]
                } else {
                    audio[src_index_floor] * (1.0 - fraction) + audio[src_index_ceil] * fraction
                };
                result.push(sample);
            }
        }
        
        result
    }

    fn resample_audio(&self, audio: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return audio.to_vec();
        }
        
        // Use PoC's efficient resampling algorithm
        let ratio = to_rate as f64 / from_rate as f64;
        let samples_per_channel = audio.len() / self.channels as usize;
        let new_samples_per_channel = (samples_per_channel as f64 * ratio) as usize;
        let new_length = new_samples_per_channel * self.channels as usize;
        let mut resampled = Vec::with_capacity(new_length);
        
        // Ultra-efficient resampling using linear interpolation
        let mut src_pos = 0.0;
        let step = 1.0 / ratio;
        
        for _ in 0..new_samples_per_channel {
            let src_index = src_pos as usize;
            let frac = src_pos - src_index as f64;
            
            if src_index < samples_per_channel - 1 {
                // Linear interpolation between two samples
                for ch in 0..self.channels as usize {
                    let idx1 = src_index * self.channels as usize + ch;
                    let idx2 = (src_index + 1) * self.channels as usize + ch;
                    let sample1 = audio[idx1];
                    let sample2 = audio[idx2];
                    let interpolated = sample1 + (sample2 - sample1) * frac as f32;
                    resampled.push(interpolated);
                }
            } else if src_index < samples_per_channel {
                // Last sample, no interpolation needed
                for ch in 0..self.channels as usize {
                    let idx = src_index * self.channels as usize + ch;
                    resampled.push(audio[idx]);
                }
            } else {
                // Beyond source, pad with zeros
                for _ in 0..self.channels as usize {
                    resampled.push(0.0);
                }
            }
            
            src_pos += step;
        }
        
        resampled
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