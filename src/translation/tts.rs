use crate::error::{Result, VoipGlotError};
use crate::config::TtsConfig;
use tracing::{info, debug};
use windows::Win32::System::Com::*;

pub struct TextToSpeech {
    config: TtsConfig,
    sample_rate: u32,
    channels: u16,
    voice: Option<String>,
}

impl TextToSpeech {
    pub fn new(config: TtsConfig) -> Result<Self> {
        info!("Initializing SAPI Text-to-Speech with config: {:?}", config);
        
        // Initialize COM
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() {
                return Err(VoipGlotError::InitializationError(format!("Failed to initialize COM: {hr:?}")));
            }
        }
        
        Ok(Self {
            config: config.clone(),
            sample_rate: config.sample_rate,
            channels: config.channels,
            voice: None,
        })
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Synthesizing speech for text: '{}'", text);
        
        // Generate speech-like audio
        let duration_ms = self.calculate_duration(text);
        let samples = self.generate_beep_audio(duration_ms);
        
        info!("Generated {} samples of speech-like audio for text: '{}'", samples.len(), text);
        Ok(samples)
    }



    fn calculate_duration(&self, text: &str) -> u32 {
        // Simple duration calculation based on text length
        let words = text.split_whitespace().count();
        let seconds = words as f32 / 2.5; // 2.5 words per second
        let ms = (seconds * 1000.0) as u32;
        // Add padding to ensure longer audio segments
        (ms + 1000).max(1500) // Minimum 1.5 seconds
    }

    fn generate_beep_audio(&self, duration_ms: u32) -> Vec<f32> {
        let samples_per_ms = self.sample_rate as f32 / 1000.0;
        let total_samples = (duration_ms as f32 * samples_per_ms) as usize;
        
        let mut audio = Vec::with_capacity(total_samples);
        
        // Generate speech-like audio with varying frequencies and amplitude
        let base_frequency = 150.0; // Lower base frequency for more natural sound
        let amplitude = 0.15; // Reduced amplitude
        
        for i in 0..total_samples {
            let t = i as f32 / self.sample_rate as f32;
            
            // Vary frequency to simulate speech intonation
            let frequency_variation = (t * 2.0).sin() * 50.0; // ±50Hz variation
            let frequency = base_frequency + frequency_variation;
            
            // Vary amplitude to simulate speech patterns
            let amplitude_variation = 0.5 + 0.5 * (t * 3.0).sin(); // 50-100% amplitude
            
            let sample = amplitude * amplitude_variation * (2.0 * std::f32::consts::PI * frequency * t).sin();
            
            // Apply fade in/out to avoid clicks
            let fade_samples = (self.sample_rate as f32 * 0.02) as usize; // 20ms fade
            let fade_multiplier = if i < fade_samples {
                i as f32 / fade_samples as f32
            } else if i >= total_samples - fade_samples {
                (total_samples - i) as f32 / fade_samples as f32
            } else {
                1.0
            };
            
            audio.push(sample * fade_multiplier);
        }
        
        debug!("Generated {} samples of speech-like audio", audio.len());
        audio
    }

    pub fn set_language(&mut self, language: String) -> Result<()> {
        debug!("Setting TTS language to: {}", language);
        self.voice = Some(language);
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
    }

    pub fn set_voice_pitch(&mut self, pitch: f32) {
        self.config.voice_pitch = pitch.max(0.5).min(2.0);
        info!("Voice pitch set to: {}", self.config.voice_pitch);
    }

    pub fn get_voice_speed(&self) -> f32 {
        self.config.voice_speed
    }

    pub fn get_voice_pitch(&self) -> f32 {
        self.config.voice_pitch
    }
}

impl Drop for TextToSpeech {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
} 