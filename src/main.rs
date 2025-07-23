//! # VoipGlot Windows Application
//! 
//! This is the Windows-specific application that uses the VoipGlot Core library
//! to provide real-time audio translation for Windows gaming and VOIP applications.
//! 
//! This application demonstrates how to integrate the voipglot-core library
//! into a Windows-specific application with proper configuration and logging.

use anyhow::Result;
use clap::Parser;
use tracing::{error, info, warn};
use tracing_subscriber;
use voipglot_core::{VoipGlotPipeline, PipelineConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

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
    
    // Initialize logging
    init_logging(args.debug)?;
    
    info!("Starting VoipGlot Windows Application");
    info!("Using VoipGlot Core library for audio processing and translation");
    
    // List audio devices if requested
    if args.list_devices {
        list_audio_devices()?;
        return Ok(());
    }
    
    // Load configuration
    let mut config = match PipelineConfig::load(&args.config) {
        Ok(config) => {
            info!("Configuration loaded successfully from {}", args.config);
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            info!("Using default configuration");
            PipelineConfig::default()
        }
    };
    
    // Override config with command line arguments
    if let Some(source_lang) = args.source_lang {
        config.translation.source_language = source_lang;
        info!("Source language set to: {}", config.translation.source_language);
    }
    if let Some(target_lang) = args.target_lang {
        config.translation.target_language = target_lang;
        info!("Target language set to: {}", config.translation.target_language);
    }
    if let Some(sample_rate) = args.sample_rate {
        config.audio.input.sample_rate = sample_rate;
        config.audio.output.sample_rate = sample_rate;
        info!("Sample rate set to: {} Hz", sample_rate);
    }
    if let Some(channels) = args.channels {
        config.audio.input.channels = channels;
        config.audio.output.channels = channels;
        info!("Audio channels set to: {}", channels);
    }
    if let Some(buffer_size) = args.buffer_size {
        config.audio.input.buffer_size = buffer_size;
        config.audio.output.buffer_size = buffer_size;
        info!("Buffer size set to: {} samples", buffer_size);
    }
    if let Some(latency_ms) = args.latency_ms {
        config.audio.input.latency_ms = latency_ms;
        config.audio.output.latency_ms = latency_ms;
        info!("Latency set to: {} ms", latency_ms);
    }
    if let Some(silence_threshold) = args.silence_threshold {
        config.processing.silence_threshold = silence_threshold;
        info!("Silence threshold set to: {}", silence_threshold);
    }
    if let Some(chunk_duration_ms) = args.chunk_duration_ms {
        config.processing.chunk_duration_ms = chunk_duration_ms;
        info!("Chunk duration set to: {} ms", chunk_duration_ms);
    }
    
    // Log pipeline configuration
    info!("Pipeline configuration:");
    info!("  STT Provider: {}", config.stt.provider);
    info!("  STT Model: {}", config.stt.model_path);
    info!("  Translation Provider: {}", config.translation.provider);
    info!("  Translation Model: {}", config.translation.model_path);
    info!("  TTS Provider: {}", config.tts.provider);
    info!("  TTS Model: {}", config.tts.model_path);
    info!("  Language Pair: {} → {}", config.translation.source_language, config.translation.target_language);
    
    // Create pipeline using core library
    let mut pipeline = match VoipGlotPipeline::new(config) {
        Ok(pipeline) => {
            info!("Pipeline created successfully");
            pipeline
        }
        Err(e) => {
            error!("Failed to create pipeline: {}", e);
            return Err(e.into());
        }
    };
    
    // Setup graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        info!("Received interrupt signal, shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;
    
    // Start processing
    info!("Starting pipeline...");
    if let Err(e) = pipeline.start().await {
        error!("Failed to start pipeline: {}", e);
        return Err(e.into());
    }
    
    info!("Pipeline started successfully. Press Ctrl+C to stop.");
    info!("VoipGlot Windows is running with real-time audio translation.");
    
    // Keep running until interrupted
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Stop processing
    info!("Stopping pipeline...");
    if let Err(e) = pipeline.stop().await {
        error!("Failed to stop pipeline: {}", e);
        return Err(e.into());
    }
    
    info!("Pipeline stopped successfully");
    info!("VoipGlot Windows shutdown complete.");
    Ok(())
}

fn init_logging(debug_flag: bool) -> Result<()> {
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
        "info"
    };
    
    // Create file layer for Windows application
    let log_file = "voipglot-win.log";
    
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
    info!("Debug mode: {}", debug_flag);
    
    Ok(())
}

fn list_audio_devices() -> Result<()> {
    use voipglot_core::audio;
    
    info!("Listing available audio devices...");
    
    // List input devices
    if let Ok(devices) = audio::list_input_devices() {
        info!("Input devices:");
        for device in devices {
            info!("  - {}", device.name);
        }
    }
    
    // List output devices
    if let Ok(devices) = audio::list_output_devices() {
        info!("Output devices:");
        for device in devices {
            info!("  - {}", device.name);
        }
    }
    
    Ok(())
} 