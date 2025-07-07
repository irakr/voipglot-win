use crate::error::{Result, VoipGlotError};
use tracing::{info, debug, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use lazy_static::lazy_static;
use rust_bert::marian::MarianGenerator;
use rust_bert::pipelines::generation_utils::GenerateConfig;
use rust_bert::pipelines::generation_utils::LanguageGenerator;


// MarianMT model manager for local translation
pub struct LocalTranslator {
    models: Arc<Mutex<HashMap<String, MarianMTModel>>>,
    cache_dir: String,
}

struct MarianMTModel {
    pipeline: Arc<MarianGenerator>,
    source_lang: String,
    target_lang: String,
}

impl LocalTranslator {
    pub fn new() -> Result<Self> {
        info!("Initializing Local Translator with MarianMT models");
        
        // Create cache directory for models
        let cache_dir = std::env::var("VOIPGLOT_MODEL_CACHE")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                format!("{}/.voipglot/models", home)
            });
        
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| VoipGlotError::Configuration(format!("Failed to create model cache directory: {}", e)))?;
        
        info!("Model cache directory: {}", cache_dir);
        
        Ok(Self {
            models: Arc::new(Mutex::new(HashMap::new())),
            cache_dir,
        })
    }

    /// Pre-load the most common translation model to avoid delays during real-time processing
    pub async fn preload_common_model(&self, source_lang: &str, target_lang: &str) -> Result<()> {
        info!("Pre-loading common translation model for {} -> {}", source_lang, target_lang);
        
        // Try to load the model upfront
        match self.get_or_load_model(source_lang, target_lang).await {
            Ok(_) => {
                info!("Successfully pre-loaded translation model for {} -> {}", source_lang, target_lang);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to pre-load translation model for {} -> {}: {}. Will load on-demand.", source_lang, target_lang, e);
                Ok(()) // Don't fail initialization, just warn
            }
        }
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
        
        debug!("Local translation: '{}' from {} to {}", text, source_lang, target_lang);
        
        // Get or load the appropriate model
        let model = self.get_or_load_model(source_lang, target_lang).await?;
        
        // Perform translation
        let output = model.pipeline.generate(Some(&[text.to_string()]), None);
        if let Some(generated) = output.first() {
            let translation = generated.text.clone();
            debug!("Local translation result: '{}'", translation);
            Ok(translation)
        } else {
            Err(VoipGlotError::Translation("No translation result returned".to_string()))
        }
    }

    async fn get_or_load_model(
        &self,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<MarianMTModel> {
        let model_key = format!("{}-{}", source_lang, target_lang);
        {
            let models = self.models.lock().await;
            if let Some(model) = models.get(&model_key) {
                debug!("Using cached model for {}", model_key);
                return Ok(MarianMTModel {
                    pipeline: Arc::clone(&model.pipeline),
                    source_lang: model.source_lang.clone(),
                    target_lang: model.target_lang.clone(),
                });
            }
        }
        info!("Loading MarianMT model for {} -> {}", source_lang, target_lang);
        let model = self.load_model(source_lang, target_lang).await?;
        {
            let mut models = self.models.lock().await;
            models.insert(model_key.clone(), MarianMTModel {
                pipeline: Arc::clone(&model.pipeline),
                source_lang: model.source_lang.clone(),
                target_lang: model.target_lang.clone(),
            });
        }
        Ok(model)
    }

    async fn load_model(
        &self,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<MarianMTModel> {
        let model_name = self.get_model_name(source_lang, target_lang)?;
        info!("Loading model: {}", model_name);
        
        // Use default configuration - the rust-bert library will handle model loading
        let pipeline = MarianGenerator::new(GenerateConfig::default())
            .map_err(|e| VoipGlotError::Translation(format!("Failed to load model {}: {}", model_name, e)))?;
        
        info!("Successfully loaded model: {}", model_name);
        Ok(MarianMTModel {
            pipeline: Arc::new(pipeline),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        })
    }

    fn get_model_name(&self, source_lang: &str, target_lang: &str) -> Result<String> {
        // Map language codes to MarianMT model names
        // These are the most common and well-supported language pairs
        let model_mapping = MARIAN_MODELS.lock().unwrap();
        
        let key = format!("{}-{}", source_lang, target_lang);
        if let Some(model_name) = model_mapping.get(&key) {
            Ok(model_name.clone())
        } else {
            // Try reverse mapping
            let reverse_key = format!("{}-{}", target_lang, source_lang);
            if let Some(model_name) = model_mapping.get(&reverse_key) {
                warn!("Model not found for {} -> {}, but reverse model exists", source_lang, target_lang);
                Ok(model_name.clone())
            } else {
                Err(VoipGlotError::Translation(format!(
                    "No MarianMT model available for {} -> {}. Available models: {:?}",
                    source_lang, target_lang, model_mapping.keys().collect::<Vec<_>>()
                )))
            }
        }
    }

    pub fn get_supported_language_pairs(&self) -> Vec<(String, String)> {
        MARIAN_MODELS.lock().unwrap()
            .keys()
            .map(|key| {
                let parts: Vec<&str> = key.split('-').collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    ("en".to_string(), "es".to_string()) // fallback
                }
            })
            .collect()
    }

    pub fn is_supported(&self, source_lang: &str, target_lang: &str) -> bool {
        let key = format!("{}-{}", source_lang, target_lang);
        let reverse_key = format!("{}-{}", target_lang, source_lang);
        let models = MARIAN_MODELS.lock().unwrap();
        models.contains_key(&key) || models.contains_key(&reverse_key)
    }
}

// Predefined MarianMT model mappings for common language pairs
lazy_static! {
    static ref MARIAN_MODELS: std::sync::Mutex<HashMap<String, String>> = {
        let mut m = HashMap::new();
        
        // English-based pairs (most common)
        m.insert("en-es".to_string(), "Helsinki-NLP/opus-mt-en-es".to_string());
        m.insert("en-fr".to_string(), "Helsinki-NLP/opus-mt-en-fr".to_string());
        m.insert("en-de".to_string(), "Helsinki-NLP/opus-mt-en-de".to_string());
        m.insert("en-it".to_string(), "Helsinki-NLP/opus-mt-en-it".to_string());
        m.insert("en-pt".to_string(), "Helsinki-NLP/opus-mt-en-pt".to_string());
        m.insert("en-ru".to_string(), "Helsinki-NLP/opus-mt-en-ru".to_string());
        m.insert("en-ja".to_string(), "Helsinki-NLP/opus-mt-en-jap".to_string());
        m.insert("en-ko".to_string(), "Helsinki-NLP/opus-mt-en-kor".to_string());
        m.insert("en-zh".to_string(), "Helsinki-NLP/opus-mt-en-zh".to_string());
        m.insert("en-ar".to_string(), "Helsinki-NLP/opus-mt-en-ar".to_string());
        m.insert("en-hi".to_string(), "Helsinki-NLP/opus-mt-en-hi".to_string());
        
        // Reverse pairs
        m.insert("es-en".to_string(), "Helsinki-NLP/opus-mt-es-en".to_string());
        m.insert("fr-en".to_string(), "Helsinki-NLP/opus-mt-fr-en".to_string());
        m.insert("de-en".to_string(), "Helsinki-NLP/opus-mt-de-en".to_string());
        m.insert("it-en".to_string(), "Helsinki-NLP/opus-mt-it-en".to_string());
        m.insert("pt-en".to_string(), "Helsinki-NLP/opus-mt-pt-en".to_string());
        m.insert("ru-en".to_string(), "Helsinki-NLP/opus-mt-ru-en".to_string());
        m.insert("jap-en".to_string(), "Helsinki-NLP/opus-mt-jap-en".to_string());
        m.insert("kor-en".to_string(), "Helsinki-NLP/opus-mt-kor-en".to_string());
        m.insert("zh-en".to_string(), "Helsinki-NLP/opus-mt-zh-en".to_string());
        m.insert("ar-en".to_string(), "Helsinki-NLP/opus-mt-ar-en".to_string());
        m.insert("hi-en".to_string(), "Helsinki-NLP/opus-mt-hi-en".to_string());
        
        // Cross-language pairs (some common ones)
        m.insert("es-fr".to_string(), "Helsinki-NLP/opus-mt-es-fr".to_string());
        m.insert("fr-es".to_string(), "Helsinki-NLP/opus-mt-fr-es".to_string());
        m.insert("de-fr".to_string(), "Helsinki-NLP/opus-mt-de-fr".to_string());
        m.insert("fr-de".to_string(), "Helsinki-NLP/opus-mt-fr-de".to_string());
        
        std::sync::Mutex::new(m)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_translator_creation() {
        let translator = LocalTranslator::new();
        assert!(translator.is_ok());
    }

    #[tokio::test]
    async fn test_supported_languages() {
        let translator = LocalTranslator::new().unwrap();
        let pairs = translator.get_supported_language_pairs();
        assert!(!pairs.is_empty());
        
        // Check that English-Spanish is supported
        assert!(translator.is_supported("en", "es"));
    }
} 