use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber;
use cpal::traits::{HostTrait, DeviceTrait};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

mod audio;
mod translation;
mod config;
mod error;

use audio::AudioManager;
use config::AppConfig;

use translation::TranslationPipeline;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// List available audio devices
    #[arg(long)]
    list_devices: bool,
    
    /// Source language for speech recognition
    #[arg(short, long)]
    source_lang: Option<String>,
    
    /// Target language for translation
    #[arg(short, long)]
    target_lang: Option<String>,
    
    /// Audio sample rate (Hz) - must be supported by your audio device
    #[arg(long)]
    sample_rate: Option<u32>,
    
    /// Audio channels (1 for mono, 2 for stereo)
    #[arg(long)]
    channels: Option<u16>,
    
    /// Audio buffer size in samples
    #[arg(long)]
    buffer_size: Option<usize>,
    
    /// Target latency in milliseconds
    #[arg(long)]
    latency_ms: Option<u32>,
    
    /// Silence threshold for voice detection
    #[arg(long)]
    silence_threshold: Option<f32>,
    
    /// Chunk duration in milliseconds
    #[arg(long)]
    chunk_duration_ms: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Load configuration
    let mut config = AppConfig::load(&args.config)?;
    
    // Override config with command line arguments
    if let Some(source_lang) = args.source_lang {
        config.translation.source_language = source_lang;
    }
    if let Some(target_lang) = args.target_lang {
        config.translation.target_language = target_lang;
    }
    if let Some(sample_rate) = args.sample_rate {
        config.audio.sample_rate = sample_rate;
    }
    if let Some(channels) = args.channels {
        config.audio.channels = channels;
    }
    if let Some(buffer_size) = args.buffer_size {
        config.audio.buffer_size = buffer_size;
    }
    if let Some(latency_ms) = args.latency_ms {
        config.audio.latency_ms = latency_ms;
    }
    if let Some(silence_threshold) = args.silence_threshold {
        config.processing.silence_threshold = silence_threshold;
    }
    if let Some(chunk_duration_ms) = args.chunk_duration_ms {
        config.processing.chunk_duration_ms = chunk_duration_ms;
    }
    
    // Initialize logging
    init_logging(&config, args.debug)?;
    
    info!("VoipGlot starting up...");
    info!("Configuration loaded from: {}", args.config);
    
    // List audio devices if requested
    if args.list_devices {
        list_audio_devices()?;
        return Ok(());
    }
    
    // Initialize components
    let mut audio_manager = AudioManager::new(config.clone());
    let mut translation_pipeline = TranslationPipeline::new(config.clone())?;
    
    // Start the pipeline
    info!("Starting VoipGlot pipeline...");
    
    audio_manager.start()?;
    translation_pipeline.start()?;
    
    info!("VoipGlot is running. Press Ctrl+C to stop.");
    info!("STT module is active - speak into your microphone to test transcription.");
    
    // Wait for interrupt signal
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        info!("Received interrupt signal, shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;
    
    // Main loop
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Shutdown
    info!("Shutting down VoipGlot...");
    translation_pipeline.stop();
    audio_manager.stop();
    
    info!("VoipGlot shutdown complete.");
    Ok(())
}

fn init_logging(config: &AppConfig, debug_flag: bool) -> Result<()> {
    use tracing_subscriber::fmt;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;
    use std::fs;
    
    // Static variable to hold the guard for the entire application lifetime
    static GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();
    
    // Determine log level
    let log_level = if debug_flag {
        "debug"
    } else {
        &config.logging.level
    };
    
    // Create file layer if configured
    if let Some(log_file) = &config.logging.log_file {
        // Remove existing log file to start fresh
        if fs::metadata(log_file).is_ok() {
            if let Err(e) = fs::remove_file(log_file) {
                eprintln!("Warning: Failed to remove existing log file '{}': {}", log_file, e);
            }
        }
        
        // Initialize with both console and file output
        let file_appender = tracing_appender::rolling::never("", log_file);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        
        // Store the guard in the static variable
        GUARD.set(guard).expect("Failed to set logging guard");
        
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level));
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_ansi(false))
            .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
            .init();
        
        info!("Logging initialized with console and file output: {}", log_file);
    } else {
        // Console only
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level));
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_ansi(false))
            .init();
        
        info!("Logging initialized with console output only");
    }
    
    Ok(())
}

fn list_audio_devices() -> Result<()> {
    info!("Available audio devices:");
    
    let host = cpal::default_host();
    
    // List input devices
    info!("Input devices:");
    for device in host.input_devices()? {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("  - {}", name);
        
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                info!("    {}Hz, {} channels, {:?}", 
                      config.max_sample_rate().0, 
                      config.channels(), 
                      config.sample_format());
            }
        }
    }
    
    // List output devices
    info!("Output devices:");
    for device in host.output_devices()? {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("  - {}", name);
        
        if let Ok(configs) = device.supported_output_configs() {
            for config in configs {
                info!("    {}Hz, {} channels, {:?}", 
                      config.max_sample_rate().0, 
                      config.channels(), 
                      config.sample_format());
            }
        }
    }
    
    Ok(())
} 