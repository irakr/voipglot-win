pub mod stt;
pub mod translator_api;
pub mod tts;
pub mod local_translator;

use crate::error::Result;
use tracing::{info, error, debug, warn};

pub use stt::SpeechToText;
pub use translator_api::TranslationApi;
pub use tts::TextToSpeech;
pub use local_translator::LocalTranslator;

pub struct Translator {
    source_language: String,
    target_language: String,
    stt: SpeechToText,
    translator: TranslationApi,
    tts: TextToSpeech,
}

impl Translator {
    pub fn new(source_lang: String, target_lang: String) -> Result<Self> {
        info!("Initializing Translator: {} -> {}", source_lang, target_lang);
        let stt = SpeechToText::new(source_lang.clone())?;
        let translator = TranslationApi::new()?;
        let tts = TextToSpeech::new(target_lang.clone())?;
        Ok(Self {
            source_language: source_lang,
            target_language: target_lang,
            stt,
            translator,
            tts,
        })
    }

    /// Pre-initialize all AI models before starting the audio pipeline
    /// This ensures all models are loaded and ready before real-time processing begins
    pub async fn initialize_models(&mut self) -> Result<()> {
        info!("Pre-initializing AI models for real-time translation...");
        
        // Pre-load translation model
        info!("Pre-loading translation model for {} -> {}", self.source_language, self.target_language);
        match self.translator.preload_model(&self.source_language, &self.target_language).await {
            Ok(_) => info!("Translation model loaded successfully"),
            Err(e) => {
                warn!("Translation model pre-load failed: {}. Will use fallback.", e);
            }
        }
        
        // Pre-load TTS model
        info!("Pre-loading TTS model for language: {}", self.target_language);
        match self.tts.initialize().await {
            Ok(_) => info!("TTS model initialized successfully"),
            Err(e) => {
                warn!("TTS model pre-load failed: {}. Will use fallback.", e);
            }
        }
        
        // Pre-load STT model (Whisper) - download if needed
        info!("Pre-loading STT model for language: {}", self.source_language);
        match self.stt.download_and_load_model().await {
            Ok(_) => info!("STT model downloaded and loaded successfully"),
            Err(e) => {
                error!("STT model download/load failed: {}. Cannot proceed without Whisper.", e);
                return Err(e);
            }
        }
        
        info!("AI model initialization completed");
        Ok(())
    }

    /// Check if all required models are ready for real-time processing
    pub fn are_models_ready(&self) -> bool {
        // For now, we assume models are ready if initialization completed
        // In the future, we can add more sophisticated checks
        true
    }

    pub async fn speech_to_text(&self, audio_data: Vec<f32>) -> Result<String> {
        debug!("Converting speech to text");
        self.stt.transcribe(audio_data).await
    }

    pub async fn translate_text(&self, text: &str) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }
        debug!("Translating text: '{}'", text);
        self.translator.translate(text, &self.source_language, &self.target_language).await
    }

    pub async fn text_to_speech(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        debug!("Converting text to speech: '{}'", text);
        self.tts.synthesize(text).await
    }

    pub fn set_source_language(&mut self, lang: String) {
        self.source_language = lang.clone();
        if let Err(e) = self.stt.set_language(lang) {
            error!("Failed to set STT language: {}", e);
        }
    }

    pub fn set_target_language(&mut self, lang: String) {
        self.target_language = lang.clone();
        if let Err(e) = self.tts.set_language(lang) {
            error!("Failed to set TTS language: {}", e);
        }
    }

    pub fn get_supported_languages(&self) -> SupportedLanguages {
        SupportedLanguages {
            stt: self.stt.get_supported_languages(),
            translation: self.translator.get_supported_languages(),
            tts: self.tts.get_supported_languages(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SupportedLanguages {
    pub stt: Vec<String>,
    pub translation: Vec<String>,
    pub tts: Vec<String>,
}

// Language code utilities
pub mod language_codes {
    use std::collections::HashMap;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref LANGUAGE_CODES: HashMap<&'static str, &'static str> = {
            let mut m = HashMap::new();
            m.insert("english", "en");
            m.insert("spanish", "es");
            m.insert("french", "fr");
            m.insert("german", "de");
            m.insert("italian", "it");
            m.insert("portuguese", "pt");
            m.insert("russian", "ru");
            m.insert("japanese", "ja");
            m.insert("korean", "ko");
            m.insert("chinese", "zh");
            m.insert("arabic", "ar");
            m.insert("hindi", "hi");
            m
        };
    }

    pub fn get_language_code(language: &str) -> Option<&str> {
        LANGUAGE_CODES.get(language.to_lowercase().as_str()).copied()
    }

    pub fn normalize_language_code(code: &str) -> String {
        code.to_lowercase()
    }
} 