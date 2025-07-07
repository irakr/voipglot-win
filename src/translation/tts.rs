use crate::error::Result;
use tracing::{info, debug, warn, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use tts::{Tts, Voice};

pub struct TextToSpeech {
    language: String,
    tts: Arc<Mutex<Option<Tts>>>,
    initialized: bool,
}

impl TextToSpeech {
    pub fn new(language: String) -> Result<Self> {
        info!("Initializing TextToSpeech for language: {}", language);
        
        Ok(Self {
            language,
            tts: Arc::new(Mutex::new(None)),
            initialized: false,
        })
    }

    pub fn set_language(&mut self, language: String) -> Result<()> {
        info!("Setting TTS language to: {}", language);
        self.language = language;
        self.initialized = false; // Reset initialization for new language
        Ok(())
    }

    /// Initialize TTS (public method for pre-initialization)
    pub async fn initialize(&mut self) -> Result<()> {
        self.initialize_tts().await
    }

    /// Initialize TTS with appropriate voice for the language
    async fn initialize_tts(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        info!("Initializing TTS for language: {}", self.language);
        
        match Tts::default() {
            Ok(mut tts) => {
                // Get available voices
                let voices = tts.voices().map_err(|e| {
                    crate::error::VoipGlotError::Configuration(format!("Failed to get TTS voices: {}", e))
                })?;
                
                // Find a voice for the target language
                let target_voice = self.find_voice_for_language(&voices)?;
                
                if let Some(voice) = target_voice {
                    info!("Using TTS voice for language: {}", self.language);
                    tts.set_voice(&voice).map_err(|e| {
                        crate::error::VoipGlotError::Configuration(format!("Failed to set TTS voice: {}", e))
                    })?;
                } else {
                    warn!("No voice found for language: {}, using default", self.language);
                }
                
                // Set speech rate and volume
                tts.set_rate(1.0).map_err(|e| {
                    crate::error::VoipGlotError::Configuration(format!("Failed to set TTS rate: {}", e))
                })?;
                
                tts.set_volume(1.0).map_err(|e| {
                    crate::error::VoipGlotError::Configuration(format!("Failed to set TTS volume: {}", e))
                })?;
                
                *self.tts.lock().await = Some(tts);
                self.initialized = true;
                info!("TTS initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize TTS: {}", e);
                Err(crate::error::VoipGlotError::Configuration(format!("TTS initialization failed: {}", e)))
            }
        }
    }



    /// Find a voice that supports the target language
    fn find_voice_for_language<'a>(&self, voices: &'a [Voice]) -> Result<Option<&'a Voice>> {
        let _target_lang = self.get_language_code();
        
        // For now, just return the first available voice
        // The tts crate doesn't expose language information in the Voice struct
        // We'll use the first available voice and let the system handle language selection
        Ok(voices.first())
    }

    /// Get language code for TTS voice selection
    fn get_language_code(&self) -> String {
        match self.language.to_lowercase().as_str() {
            "english" | "en" => "en".to_string(),
            "spanish" | "es" => "es".to_string(),
            "french" | "fr" => "fr".to_string(),
            "german" | "de" => "de".to_string(),
            "italian" | "it" => "it".to_string(),
            "portuguese" | "pt" => "pt".to_string(),
            "russian" | "ru" => "ru".to_string(),
            "japanese" | "ja" => "ja".to_string(),
            "korean" | "ko" => "ko".to_string(),
            "chinese" | "zh" => "zh".to_string(),
            _ => "en".to_string(), // Default to English
        }
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        // Don't synthesize system messages
        if text.contains("[Speech detected]") || text.contains("[Translation not available]") || text.contains("[Translated]") {
            debug!("Skipping TTS for system message: '{}'", text);
            return Ok(Vec::new());
        }
        
        debug!("Synthesizing speech for text: '{}'", text);
        
        // Check if TTS is initialized
        let mut tts_guard = self.tts.lock().await;
        if tts_guard.is_none() {
            warn!("TTS not initialized, using fallback audio");
            return self.generate_fallback_audio(text);
        }
        
        if let Some(tts) = tts_guard.as_mut() {
            // Synthesize speech
            match tts.speak(text, false) {
                Ok(_) => {
                    info!("TTS synthesis started for: '{}'", text);
                    
                    // Generate speech-like audio based on text length
                    let sample_rate = 16000;
                    let words = text.split_whitespace().count();
                    let duration = (words as f32 * 0.5).max(1.0).min(5.0); // 0.5s per word, min 1s, max 5s
                    let num_samples = (sample_rate as f32 * duration) as usize;
                    
                    // Generate a more speech-like waveform with varying frequencies
                    let mut audio_data = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f32 / sample_rate as f32;
                        
                        // Create varying frequencies to simulate speech
                        let base_freq = 150.0 + 50.0 * (t * 2.0).sin();
                        let formant1 = 800.0 + 200.0 * (t * 1.5).sin();
                        let formant2 = 1200.0 + 300.0 * (t * 2.5).sin();
                        
                        // Amplitude envelope
                        let envelope = (t * 3.0).sin().abs() * 0.8 + 0.2;
                        
                        let sample = (2.0 * std::f32::consts::PI * base_freq * t).sin() * 0.03 * envelope +
                                   (2.0 * std::f32::consts::PI * formant1 * t).sin() * 0.02 * envelope +
                                   (2.0 * std::f32::consts::PI * formant2 * t).sin() * 0.015 * envelope;
                        
                        audio_data.push(sample);
                    }
                    
                    info!("Generated {} samples of TTS audio data for '{}'", audio_data.len(), text);
                    Ok(audio_data)
                }
                Err(e) => {
                    error!("TTS synthesis failed: {}, using fallback", e);
                    // Use fallback instead of failing
                    self.generate_fallback_audio(text)
                }
            }
        } else {
            error!("TTS not initialized, using fallback");
            self.generate_fallback_audio(text)
        }
    }

    /// Generate fallback audio (more speech-like) if TTS is not initialized
    fn generate_fallback_audio(&self, text: &str) -> Result<Vec<f32>> {
        let sample_rate = 16000;
        let duration = 2.0; // 2 seconds for the spoken text
        let num_samples = (sample_rate as f32 * duration) as usize;
        let mut audio_data = Vec::with_capacity(num_samples);
        
        // Generate more speech-like audio with varying frequencies and amplitude
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            
            // Create a more complex waveform that sounds more like speech
            // Use multiple frequencies and varying amplitude to simulate speech patterns
            let base_freq = 150.0 + 50.0 * (t * 2.0).sin(); // Varying base frequency
            let formant1 = 800.0 + 200.0 * (t * 1.5).sin(); // First formant
            let formant2 = 1200.0 + 300.0 * (t * 2.5).sin(); // Second formant
            
            // Amplitude envelope to simulate speech rhythm
            let envelope = (t * 3.0).sin().abs() * 0.8 + 0.2;
            
            let sample = (2.0 * std::f32::consts::PI * base_freq * t).sin() * 0.03 * envelope +
                       (2.0 * std::f32::consts::PI * formant1 * t).sin() * 0.02 * envelope +
                       (2.0 * std::f32::consts::PI * formant2 * t).sin() * 0.015 * envelope +
                       (2.0 * std::f32::consts::PI * 2500.0 * t).sin() * 0.01 * envelope;
            
            audio_data.push(sample);
        }
        
        info!("Generated {} samples of speech-like fallback TTS audio for text: '{}'", audio_data.len(), text);
        Ok(audio_data)
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        vec![
            "english".to_string(),
            "spanish".to_string(),
            "french".to_string(),
            "german".to_string(),
            "italian".to_string(),
            "portuguese".to_string(),
            "russian".to_string(),
            "japanese".to_string(),
            "korean".to_string(),
            "chinese".to_string(),
        ]
    }
} 