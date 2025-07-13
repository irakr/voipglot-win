pub mod stt;
pub mod translator_api;
pub mod tts;

use crate::error::{Result, VoipGlotError};
use crate::config::{SttConfig, TranslationConfig, TtsConfig};
use tracing::{info, error, debug};

pub use stt::SpeechToText;
pub use translator_api::TranslationApi;
pub use tts::TextToSpeech;

pub struct Translator {
    source_language: String,
    target_language: String,
    stt: SpeechToText,
    translator: TranslationApi,
    tts: TextToSpeech,
}

impl Translator {
    pub fn new(stt_config: SttConfig, translation_config: TranslationConfig, tts_config: TtsConfig) -> Result<Self> {
        info!("Initializing Translator: {} -> {}", translation_config.source_language, translation_config.target_language);
        
        let stt = SpeechToText::new(stt_config)?;
        let translator = TranslationApi::new(translation_config.clone())?;
        let tts = TextToSpeech::new(tts_config)?;
        
        Ok(Self {
            source_language: translation_config.source_language.clone(),
            target_language: translation_config.target_language.clone(),
            stt,
            translator,
            tts,
        })
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

    pub async fn process_audio_pipeline(&self, audio_data: Vec<f32>) -> Result<Option<Vec<f32>>> {
        debug!("Processing audio pipeline with {} samples", audio_data.len());
        
        // Step 1: Speech to Text
        let transcribed_text = self.speech_to_text(audio_data).await?;
        if transcribed_text.is_empty() {
            debug!("No speech detected, skipping translation");
            return Ok(None);
        }
        
        info!("Transcribed: '{}'", transcribed_text);
        
        // Step 2: Translation
        let translated_text = self.translate_text(&transcribed_text).await?;
        if translated_text.is_empty() {
            debug!("Translation failed or produced empty result");
            return Ok(None);
        }
        
        info!("Translated: '{}' -> '{}'", transcribed_text, translated_text);
        
        // Step 3: Text to Speech
        let synthesized_audio = self.text_to_speech(&translated_text).await?;
        if synthesized_audio.is_empty() {
            debug!("TTS failed or produced empty audio");
            return Ok(None);
        }
        
        info!("Synthesized {} samples of audio", synthesized_audio.len());
        Ok(Some(synthesized_audio))
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

    pub fn get_source_language(&self) -> &str {
        &self.source_language
    }

    pub fn get_target_language(&self) -> &str {
        &self.target_language
    }

    pub fn reset_stt(&mut self) -> Result<()> {
        debug!("Resetting STT recognizer");
        self.stt.reset()
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