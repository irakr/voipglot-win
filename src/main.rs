use anyhow::Result;
use clap::Parser;
use tracing::{info, error, Level};
use tracing_subscriber;

mod audio;
mod translation;
mod config;
mod error;

use audio::AudioManager;
use config::AppConfig;
use error::VoipGlotError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Source language for speech recognition
    #[arg(short, long, default_value = "en")]
    source_lang: String,
    
    /// Target language for translation
    #[arg(short, long, default_value = "es")]
    target_lang: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.debug { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    
    info!("Starting VoipGlot Windows Audio Translation App");
    info!("Source language: {}", args.source_lang);
    info!("Target language: {}", args.target_lang);
    
    // Load configuration
    let config = AppConfig::load(&args.config)?;
    info!("Configuration loaded successfully");
    
    // Initialize audio manager
    let audio_manager = AudioManager::new(config.audio.clone())?;
    info!("Audio manager initialized");
    
    // Start the audio processing pipeline
    match run_audio_pipeline(audio_manager, args.source_lang, args.target_lang).await {
        Ok(_) => {
            info!("Audio pipeline completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Audio pipeline failed: {}", e);
            Err(e)
        }
    }
}

async fn run_audio_pipeline(
    audio_manager: AudioManager,
    source_lang: String,
    target_lang: String,
) -> Result<()> {
    info!("Starting audio processing pipeline");
    
    // Initialize translation components
    let translator = translation::Translator::new(source_lang, target_lang)?;
    info!("Translation engine initialized");
    
    // Start audio capture and processing
    audio_manager.start_processing(translator).await?;
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal");
    
    Ok(())
} 