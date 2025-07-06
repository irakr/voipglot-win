pub mod stt;
pub mod translator_api;
pub mod tts;
pub mod local_translator;

use crate::error::Result;
use tracing::{info, error, debug};

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