use anyhow::{Context, Result};
use ct2rs::{Device, Translator, TranslationOptions, ComputeType};
use serde::Deserialize;
use std::path::PathBuf;
use std::io::{self, Write};
use tracing::{info, warn, error, Level};

#[derive(Debug, Deserialize)]
struct ModelConfig {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct TranslationConfig {
    source_lang: String,
    target_lang: String,
    num_threads: usize,
    device: String,
    max_batch_size: usize,
    beam_size: usize,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    model: ModelConfig,
    translation: TranslationConfig,
    logging: LoggingConfig,
}

fn setup_logging(level: &str) -> Result<()> {
    let level = level.parse::<Level>().context("Failed to parse log level")?;
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    Ok(())
}

fn load_config() -> Result<AppConfig> {
    let config_str = std::fs::read_to_string("config.toml")
        .context("Failed to read config.toml")?;
    toml::from_str(&config_str).context("Failed to parse config.toml")
}

fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = load_config()?;
    
    // Setup logging
    setup_logging(&config.logging.level)?;
    
    info!("Initializing CTranslate2 translator...");
    info!("Source language: {}", config.translation.source_lang);
    info!("Target language: {}", config.translation.target_lang);
    info!("Device: {}", config.translation.device);
    info!("Max batch size: {}", config.translation.max_batch_size);
    
    // Check if model path exists
    if !config.model.path.exists() {
        error!("Model path does not exist: {:?}", config.model.path);
        return Err(anyhow::anyhow!("Model path does not exist: {:?}", config.model.path));
    }
    
    info!("Model path exists: {:?}", config.model.path);
    
    // Check if model path is a directory
    if !config.model.path.is_dir() {
        error!("Model path is not a directory: {:?}", config.model.path);
        return Err(anyhow::anyhow!("Model path is not a directory: {:?}", config.model.path));
    }
    
    info!("Model path is a directory");
    
    // List files in model directory
    match std::fs::read_dir(&config.model.path) {
        Ok(entries) => {
            let files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect();
            info!("Model directory contains files: {:?}", files);
        }
        Err(e) => {
            warn!("Could not read model directory: {}", e);
        }
    }
    
    // Try to load translator with minimal configuration
    info!("Attempting to load translator with minimal configuration...");
    
    // Clone the path so we can use it multiple times
    let model_path = config.model.path.clone();
    
    // Try loading without any custom config first
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
            
            // Try with minimal custom config
            let minimal_config = ct2rs::Config {
                device: Device::CPU,
                compute_type: ComputeType::default(),
                device_indices: vec![0],
                max_queued_batches: 0,
                cpu_core_offset: 0,
                tensor_parallel: false,
                num_threads_per_replica: 1,
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
    
    info!("Translator initialized successfully");
    
    println!("\n=== CTranslate2 Translation Tool ===");
    println!("Source language: {}", config.translation.source_lang);
    println!("Target language: {}", config.translation.target_lang);
    println!("Enter text to translate (or 'quit' to exit):\n");
    
    // Interactive translation loop
    loop {
        let input = get_user_input("> ")?;
        
        if input.to_lowercase() == "quit" || input.to_lowercase() == "exit" {
            println!("Goodbye!");
            break;
        }
        
        if input.trim().is_empty() {
            continue;
        }
        
        info!("Translating: {}", input);
        
        // Create translation options using Default trait
        let mut options = TranslationOptions::default();
        options.beam_size = config.translation.beam_size;
        
        // Use translate_batch with correct API
        let translations = match translator.translate_batch(
            &[input.clone()],
            &options,
            None,  // No callback function
        ) {
            Ok(translations) => translations,
            Err(e) => {
                error!("Translation failed: {}", e);
                println!("Error: Translation failed - {}", e);
                continue;
            }
        };
        
        if let Some(translation) = translations.first() {
            // Translation result is a tuple (String, Option<f32>)
            let (translated_text, score) = translation;
            println!("Translation: {}", translated_text);
            if let Some(s) = score {
                println!("Confidence: {:.2}", s);
            }
        } else {
            warn!("No translation produced");
            println!("Error: No translation produced");
        }
        
        println!(); // Empty line for readability
    }
    
    info!("Translation tool completed");
    Ok(())
} 