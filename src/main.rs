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
    
    /// Enable audio passthrough mode (bypasses AI processing)
    #[arg(short, long)]
    passthrough: bool,
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
    let mut audio_manager = AudioManager::new(config.audio.clone(), config.processing.clone())?;
    info!("Audio manager initialized");
    
    // Enable passthrough mode if requested
    if args.passthrough {
        info!("Audio passthrough mode enabled - bypassing AI processing");
        audio_manager.enable_passthrough_mode();
    }
    
    // Start the audio processing pipeline
    match run_audio_pipeline(&mut audio_manager, args.source_lang, args.target_lang).await {
        Ok(_) => {
            info!("Audio pipeline completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Audio pipeline failed: {}", e);
            Err(e.into())
        }
    }
}

async fn run_audio_pipeline(
    audio_manager: &mut AudioManager,
    source_lang: String,
    target_lang: String,
) -> Result<()> {
    info!("Starting audio processing pipeline");
    
    // Initialize translation components
    let mut translator = translation::Translator::new(source_lang, target_lang)?;
    
    // Pre-initialize all AI models before starting audio processing
    info!("Pre-initializing AI models...");
    info!("This may take a few minutes on first run to download the Whisper model...");
    match translator.initialize_models().await {
        Ok(_) => info!("AI models initialized successfully"),
        Err(e) => {
            error!("AI model initialization failed: {}. Cannot proceed without required models.", e);
            return Err(e.into());
        }
    }
    
    info!("Translation engine initialized");
    
    // Start audio capture and processing
    audio_manager.start_processing(translator).await?;
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal");
    
    Ok(())
} 