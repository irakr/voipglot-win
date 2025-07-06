use crate::error::Result;
use tracing::{info, debug};
use super::LocalTranslator;

pub struct TranslationApi {
    local_translator: LocalTranslator,
}

impl TranslationApi {
    pub fn new() -> Result<Self> {
        info!("Initializing Translation API (local only)");
        let local_translator = LocalTranslator::new()?;
        Ok(Self { local_translator })
    }

    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }
        debug!("Translating text locally: '{}' from {} to {}", text, source_lang, target_lang);
        self.local_translator.translate(text, source_lang, target_lang).await
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        // Return supported languages from local translator
        self.local_translator.get_supported_language_pairs()
            .into_iter()
            .flat_map(|(a, b)| vec![a, b])
            .collect()
    }
} 