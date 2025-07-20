use anyhow::Result;
use ct2rs::{Device, Translator, TranslationOptions, ComputeType};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct TranslatorProcessor {
    text_tx: mpsc::UnboundedSender<String>,
    config: AppConfig,
}

impl TranslatorProcessor {
    pub fn new(config: AppConfig, text_tx: mpsc::UnboundedSender<String>) -> Result<Self> {
        info!("Initializing CTranslate2 translator");
        info!("Translation config: {} -> {}", 
              config.translation.source_language, 
              config.translation.target_language);
        
        if config.translation.source_language == config.translation.target_language {
            info!("Translation bypass mode: ENABLED (same source and target language)");
        } else {
            info!("Translation bypass mode: DISABLED (different source and target languages)");
            
            // Check if model path exists
            let model_path = &config.translation.model_path;
            if !std::path::Path::new(model_path).exists() {
                error!("Model path does not exist: {}", model_path);
                return Err(anyhow::anyhow!("Model path does not exist: {}", model_path));
            }
            
            info!("Model path exists: {}", model_path);
        }
        
        Ok(Self {
            text_tx,
            config,
        })
    }
    
    pub fn translate_text(&mut self, source_text: &str) -> Result<String> {
        if source_text.trim().is_empty() {
            return Ok(String::new());
        }
        
        // Check if source and target languages are the same
        if self.config.translation.source_language == self.config.translation.target_language {
            info!("TRANSLATION BYPASS: Source and target languages match ({}), bypassing translation", 
                  self.config.translation.source_language);
            info!("BYPASSED TEXT: \"{}\"", source_text);
            return Ok(source_text.to_string());
        }
        
        // Create translator exactly like the test app does
        let model_path = std::path::PathBuf::from(&self.config.translation.model_path);
        
        info!("Attempting to load translator with minimal configuration...");
        
        // Try loading without any custom config first (following test app pattern)
        let translator = match Translator::new(
            model_path.clone(),
            &ct2rs::Config::default(),  // Use default config
        ) {
            Ok(translator) => {
                info!("Translator loaded successfully with default config");
                translator
            }
            Err(e) => {
                error!("Failed to load translator with default config: {}", e);
                info!("Trying with minimal custom config...");
                
                // Try with minimal custom config (following test app pattern)
                let minimal_config = ct2rs::Config {
                    device: Device::CPU,
                    compute_type: ComputeType::default(),
                    device_indices: vec![0],
                    max_queued_batches: 1, // Slightly optimized for low latency
                    cpu_core_offset: 0,
                    tensor_parallel: false,
                    num_threads_per_replica: self.config.translation.num_threads.min(4),
                };
                
                match Translator::new(model_path, &minimal_config) {
                    Ok(translator) => {
                        info!("Translator loaded successfully with minimal config");
                        translator
                    }
                    Err(e2) => {
                        error!("Failed to load translator with minimal config: {}", e2);
                        return Err(anyhow::anyhow!("Failed to load translator: {} (default), {} (minimal)", e, e2));
                    }
                }
            }
        };
        
        info!("Translating text: \"{}\"", source_text);
        
        // Create translation options optimized for low latency
        let mut options = TranslationOptions::default();
        options.beam_size = self.config.translation.beam_size.min(2); // Smaller beam size for lower latency
        
        // Perform translation (exactly like test app)
        let translations = translator.translate_batch(
            &[source_text.to_string()],
            &options,
            None, // No callback function
        )?;
        
        if let Some((translated_text, score)) = translations.first() {
            info!("TRANSLATION RESULT: \"{}\" -> \"{}\"", source_text, translated_text);
            if let Some(confidence) = score {
                debug!("Translation confidence: {:.2}", confidence);
            }
            
            // Print translation to terminal for user visibility
            println!("TRANSLATED: {} -> {}", source_text, translated_text);
            
            Ok(translated_text.clone())
        } else {
            warn!("No translation produced for: \"{}\"", source_text);
            // Return original text as fallback
            Ok(source_text.to_string())
        }
    }
    
    pub fn process_translation_pipeline(&mut self, source_text: String) -> Result<()> {
        match self.translate_text(&source_text) {
            Ok(translated_text) => {
                // Send translated text to TTS module
                if let Err(e) = self.text_tx.send(translated_text) {
                    error!("Failed to send translated text: {}", e);
                }
            }
            Err(e) => {
                error!("Translation failed: {}", e);
                // Send original text as fallback
                if let Err(send_err) = self.text_tx.send(source_text) {
                    error!("Failed to send fallback text: {}", send_err);
                }
            }
        }
        Ok(())
    }
}
