use crate::error::{Result, VoipGlotError};
use crate::config::TranslationConfig;
use tracing::{info, error, debug, warn};
use ct2rs::{Device, Translator, TranslationOptions, ComputeType, tokenizers};
use std::path::PathBuf;

pub struct TranslationApi {
    config: TranslationConfig,
    translator: Option<Translator<tokenizers::auto::Tokenizer>>,
}

impl TranslationApi {
    pub fn new(config: TranslationConfig) -> Result<Self> {
        info!("Initializing CTranslate2 translation with config: {:?}", config);
        
        // Validate model path
        let model_path = PathBuf::from(&config.model_path);
        if !model_path.exists() {
            return Err(VoipGlotError::Configuration(
                format!("CT2 model path does not exist: {}", config.model_path)
            ));
        }
        
        if !model_path.is_dir() {
            return Err(VoipGlotError::Configuration(
                format!("CT2 model path is not a directory: {}", config.model_path)
            ));
        }
        
        info!("Loading CT2 model from: {:?}", model_path);
        
        // Try to load translator with minimal configuration first
        let translator = match Translator::<tokenizers::auto::Tokenizer>::new(
            model_path.clone(),
            &ct2rs::Config::default(),
        ) {
            Ok(translator) => {
                info!("Translator loaded successfully with default config");
                translator
            }
            Err(e) => {
                error!("Failed to load translator with default config: {}", e);
                info!("Trying with minimal custom config...");
                
                // Try with minimal custom config
                let minimal_config = ct2rs::Config {
                    device: Device::CPU,
                    compute_type: ComputeType::default(),
                    device_indices: vec![0],
                    max_queued_batches: 0,
                    cpu_core_offset: 0,
                    tensor_parallel: false,
                    num_threads_per_replica: config.num_threads,
                };
                
                match Translator::<tokenizers::auto::Tokenizer>::new(model_path, &minimal_config) {
                    Ok(translator) => {
                        info!("Translator loaded successfully with minimal config");
                        translator
                    }
                    Err(e2) => {
                        error!("Failed to load translator with minimal config: {}", e2);
                        return Err(VoipGlotError::Configuration(
                            format!("Failed to load CT2 translator: {} (default), {} (minimal)", e, e2)
                        ));
                    }
                }
            }
        };
        
        info!("CT2 translator initialized successfully");
        
        Ok(Self {
            config,
            translator: Some(translator),
        })
    }

    pub async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }
        
        debug!("Translating text: '{}' from {} to {}", text, source_lang, target_lang);
        
        let translator = self.translator.as_ref()
            .ok_or_else(|| VoipGlotError::Translation("Translator not initialized".to_string()))?;
        
        // Create translation options
        let mut options = TranslationOptions::default();
        options.beam_size = self.config.beam_size;
        
        // Use translate_batch with correct API
        let translations = match translator.translate_batch(
            &[text.to_string()],
            &options,
            None,  // No callback function
        ) {
            Ok(translations) => translations,
            Err(e) => {
                error!("Translation failed: {}", e);
                return Err(VoipGlotError::Translation(format!("CT2 translation failed: {}", e)));
            }
        };
        
        if let Some(translation) = translations.first() {
            // Translation result is a tuple (String, Option<f32>)
            let (translated_text, score) = translation;
            info!("Translation: '{}' -> '{}' (confidence: {:?})", text, translated_text, score);
            return Ok(translated_text.clone());
        } else {
            warn!("No translation produced for text: {}", text);
            return Err(VoipGlotError::Translation("No translation produced".to_string()));
        }
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        // NLLB-200 supports 200+ languages, but we'll list the most common ones
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
            "ar".to_string(), // Arabic
            "hi".to_string(), // Hindi
            "nl".to_string(), // Dutch
            "pl".to_string(), // Polish
            "tr".to_string(), // Turkish
            "sv".to_string(), // Swedish
            "da".to_string(), // Danish
            "no".to_string(), // Norwegian
            "fi".to_string(), // Finnish
            "cs".to_string(), // Czech
            "sk".to_string(), // Slovak
            "hu".to_string(), // Hungarian
            "ro".to_string(), // Romanian
            "bg".to_string(), // Bulgarian
            "hr".to_string(), // Croatian
            "sr".to_string(), // Serbian
            "sl".to_string(), // Slovenian
            "et".to_string(), // Estonian
            "lv".to_string(), // Latvian
            "lt".to_string(), // Lithuanian
        ]
    }

    pub fn get_model_path(&self) -> &str {
        &self.config.model_path
    }

    pub fn get_device(&self) -> &str {
        &self.config.device
    }

    pub fn get_num_threads(&self) -> usize {
        self.config.num_threads
    }
} 