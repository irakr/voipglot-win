use anyhow::Result;
use clap::Parser;
use tracing::{info, error, Level};
use tracing_subscriber;
use cpal::traits::{HostTrait, DeviceTrait};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{Write, Seek};

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
    // Set up panic hook to capture crash logs
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else {
            "Unknown panic"
        };
        
        let location = if let Some(location) = panic_info.location() {
            format!("{}:{}:{}", location.file(), location.line(), location.column())
        } else {
            "Unknown location".to_string()
        };
        
        eprintln!("PANIC: {} at {}", msg, location);
        
        // Try to write to log file if it exists
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("voipglot.log") {
            let _ = writeln!(file, "PANIC: {} at {}", msg, location);
            let _ = file.flush();
        }
    }));
    
    let args = Args::parse();
    
    // Show help if no arguments provided
    if std::env::args().count() == 1 {
        println!("VoipGlot Windows Audio Translation App");
        println!("Usage: voipglot-win [OPTIONS]");
        println!();
        println!("Examples:");
        println!("  voipglot-win                                    # Run with default config");
        println!("  voipglot-win --source-lang en --target-lang es  # English to Spanish");
        println!("  voipglot-win --sample-rate 48000               # Use 48kHz sample rate");
        println!("  voipglot-win --list-devices                    # List audio devices");
        println!();
        println!("Use --help for all available options");
        println!();
    }
    
    // Load configuration first to get logging settings
    let mut config = AppConfig::load(&args.config)?;
    
    // Validate and override configuration with command line arguments
    validate_and_override_config(&mut config, &args)?;
    
    // Initialize logging based on configuration
    init_logging(&config, args.debug)?;
    
    info!("Starting VoipGlot Windows Audio Translation App");
    info!("Source language: {}", config.translation.source_language);
    info!("Target language: {}", config.translation.target_language);
    info!("Configuration loaded successfully");
    
    // List devices if requested
    if args.list_devices {
        list_audio_devices()?;
        return Ok(());
    }
    
    // Validate VB Cable device
    validate_vb_cable_device(&config)?;
    
    // Initialize audio manager
    let mut audio_manager = AudioManager::new(config.audio.clone(), config.processing.clone())?;
    info!("Audio manager initialized");
    
    // Start the audio processing pipeline
    let result = match run_audio_pipeline(&mut audio_manager, config.clone()).await {
        Ok(_) => {
            info!("Audio pipeline completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Audio pipeline failed: {}", e);
            Err(e)
        }
    };
    
    // Ensure logs are flushed before exit
    if let Some(log_file) = &config.logging.log_file {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file) {
            let _ = writeln!(file, "Application shutting down");
            let _ = file.flush();
        }
    }
    
    result
}

fn validate_and_override_config(config: &mut AppConfig, args: &Args) -> Result<()> {
    // Validate and override language settings
    if let Some(source_lang) = &args.source_lang {
        validate_language_code(source_lang)?;
        config.translation.source_language = source_lang.clone();
    }
    
    if let Some(target_lang) = &args.target_lang {
        validate_language_code(target_lang)?;
        config.translation.target_language = target_lang.clone();
    }
    
    // Validate and override audio settings
    if let Some(sample_rate) = args.sample_rate {
        validate_sample_rate(sample_rate)?;
        config.audio.sample_rate = sample_rate;
        config.stt.sample_rate = sample_rate as f32;
        config.tts.sample_rate = sample_rate;
    }
    
    if let Some(channels) = args.channels {
        validate_channels(channels)?;
        config.audio.channels = channels;
        config.tts.channels = channels;
    }
    
    if let Some(buffer_size) = args.buffer_size {
        validate_buffer_size(buffer_size)?;
        config.audio.buffer_size = buffer_size;
    }
    
    if let Some(latency_ms) = args.latency_ms {
        validate_latency(latency_ms)?;
        config.audio.latency_ms = latency_ms;
    }
    
    if let Some(silence_threshold) = args.silence_threshold {
        validate_silence_threshold(silence_threshold)?;
        config.processing.silence_threshold = silence_threshold;
    }
    
    if let Some(chunk_duration_ms) = args.chunk_duration_ms {
        validate_chunk_duration(chunk_duration_ms)?;
        config.processing.chunk_duration_ms = chunk_duration_ms;
    }
    
    // Adapt configuration to actual device capabilities
    adapt_to_device_capabilities(config)?;
    
    Ok(())
}

fn validate_language_code(lang: &str) -> Result<()> {
    let valid_languages = ["en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh"];
    if !valid_languages.contains(&lang) {
        return Err(VoipGlotError::Configuration(
            format!("Invalid language code '{}'. Supported languages: {}", 
                   lang, valid_languages.join(", "))
        ).into());
    }
    Ok(())
}

fn validate_sample_rate(sample_rate: u32) -> Result<()> {
    let valid_rates = [8000, 16000, 22050, 32000, 44100, 48000];
    if !valid_rates.contains(&sample_rate) {
        return Err(VoipGlotError::Configuration(
            format!("Invalid sample rate {} Hz. Supported rates: {}", 
                   sample_rate, valid_rates.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", "))
        ).into());
    }
    Ok(())
}

fn validate_channels(channels: u16) -> Result<()> {
    if channels != 1 && channels != 2 {
        return Err(VoipGlotError::Configuration(
            format!("Invalid channel count {}. Must be 1 (mono) or 2 (stereo)", channels)
        ).into());
    }
    Ok(())
}

fn validate_buffer_size(buffer_size: usize) -> Result<()> {
    if buffer_size < 256 || buffer_size > 16384 {
        return Err(VoipGlotError::Configuration(
            format!("Invalid buffer size {}. Must be between 256 and 16384", buffer_size)
        ).into());
    }
    Ok(())
}

fn validate_latency(latency_ms: u32) -> Result<()> {
    if latency_ms < 10 || latency_ms > 1000 {
        return Err(VoipGlotError::Configuration(
            format!("Invalid latency {} ms. Must be between 10 and 1000", latency_ms)
        ).into());
    }
    Ok(())
}

fn validate_silence_threshold(threshold: f32) -> Result<()> {
    if threshold < 0.001 || threshold > 1.0 {
        return Err(VoipGlotError::Configuration(
            format!("Invalid silence threshold {}. Must be between 0.001 and 1.0", threshold)
        ).into());
    }
    Ok(())
}

fn validate_chunk_duration(duration_ms: u32) -> Result<()> {
    if duration_ms < 50 || duration_ms > 5000 {
        return Err(VoipGlotError::Configuration(
            format!("Invalid chunk duration {} ms. Must be between 50 and 5000", duration_ms)
        ).into());
    }
    Ok(())
}

fn adapt_to_device_capabilities(config: &mut AppConfig) -> Result<()> {
    info!("Adapting configuration to actual device capabilities");
    
    let host = cpal::default_host();
    
    // Get input device capabilities
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if let Some(configured_device) = &config.audio.input_device {
                    if name.contains(configured_device) {
                        if let Ok(supported_config) = device.default_input_config() {
                            info!("Input device '{}' supports: {:?}", name, supported_config);
                            
                            // Adapt sample rate if needed
                            let device_sample_rate = supported_config.sample_rate().0;
                            if config.audio.sample_rate != device_sample_rate {
                                info!("Adapting sample rate from {} Hz to {} Hz (device capability)", 
                                     config.audio.sample_rate, device_sample_rate);
                                config.audio.sample_rate = device_sample_rate;
                                config.stt.sample_rate = device_sample_rate as f32;
                                config.tts.sample_rate = device_sample_rate;
                            }
                            
                            // Adapt channels if needed
                            let device_channels = supported_config.channels();
                            if config.audio.channels != device_channels {
                                info!("Adapting channels from {} to {} (device capability)", 
                                     config.audio.channels, device_channels);
                                config.audio.channels = device_channels;
                                config.tts.channels = device_channels;
                            }
                            
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // Get output device capabilities
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if let Some(configured_device) = &config.audio.output_device {
                    if name.contains(configured_device) {
                        if let Ok(supported_config) = device.default_output_config() {
                            info!("Output device '{}' supports: {:?}", name, supported_config);
                            
                            // Adapt sample rate if needed
                            let device_sample_rate = supported_config.sample_rate().0;
                            if config.audio.sample_rate != device_sample_rate {
                                info!("Adapting sample rate from {} Hz to {} Hz (output device capability)", 
                                     config.audio.sample_rate, device_sample_rate);
                                config.audio.sample_rate = device_sample_rate;
                                config.stt.sample_rate = device_sample_rate as f32;
                                config.tts.sample_rate = device_sample_rate;
                            }
                            
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // If no specific device is configured, use default device capabilities
    if config.audio.input_device.is_none() {
        if let Some(default_device) = host.default_input_device() {
            if let Ok(supported_config) = default_device.default_input_config() {
                info!("Default input device supports: {:?}", supported_config);
                
                let device_sample_rate = supported_config.sample_rate().0;
                if config.audio.sample_rate != device_sample_rate {
                    info!("Adapting to default device sample rate: {} Hz", device_sample_rate);
                    config.audio.sample_rate = device_sample_rate;
                    config.stt.sample_rate = device_sample_rate as f32;
                    config.tts.sample_rate = device_sample_rate;
                }
                
                let device_channels = supported_config.channels();
                if config.audio.channels != device_channels {
                    info!("Adapting to default device channels: {}", device_channels);
                    config.audio.channels = device_channels;
                    config.tts.channels = device_channels;
                }
            }
        }
    }
    
    Ok(())
}

fn list_audio_devices() -> Result<()> {
    info!("Listing available audio devices...");
    
    let host = cpal::default_host();
    
    // List input devices
    println!("\n=== Input Devices ===");
    if let Ok(devices) = host.input_devices() {
        for (i, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                println!("  {}: {}", i, name);
            }
        }
    }
    
    // List output devices
    println!("\n=== Output Devices ===");
    if let Ok(devices) = host.output_devices() {
        for (i, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                println!("  {}: {}", i, name);
            }
        }
    }
    
    // Check for VB Cable devices
    println!("\n=== VB Cable Devices ===");
    let mut found_vb_cable = false;
    
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.contains("CABLE") || name.contains("VB-Audio") {
                    println!("  Input: {}", name);
                    found_vb_cable = true;
                }
            }
        }
    }
    
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.contains("CABLE") || name.contains("VB-Audio") {
                    println!("  Output: {}", name);
                    found_vb_cable = true;
                }
            }
        }
    }
    
    if !found_vb_cable {
        println!("  No VB Cable devices found");
        println!("  Please install VB-Audio Virtual Cable to use this application");
    }
    
    Ok(())
}

fn validate_vb_cable_device(_config: &AppConfig) -> Result<()> {
    info!("Validating VB Cable device configuration");
    
    let host = cpal::default_host();
    let mut found_vb_cable = false;
    
    // Check input devices
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.contains("CABLE") || name.contains("VB-Audio") {
                    info!("Found VB Cable input device: {}", name);
                    found_vb_cable = true;
                    break;
                }
            }
        }
    }
    
    // Check output devices
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.contains("CABLE") || name.contains("VB-Audio") {
                    info!("Found VB Cable output device: {}", name);
                    found_vb_cable = true;
                    break;
                }
            }
        }
    }
    
    if !found_vb_cable {
        error!("VB Cable device not found");
        error!("Please install VB-Audio Virtual Cable and ensure it's properly configured");
        return Err(VoipGlotError::DeviceNotFound("VB Cable device not found".to_string()).into());
    }
    
    info!("VB Cable device validation passed");
    Ok(())
}

async fn run_audio_pipeline(
    audio_manager: &mut AudioManager,
    config: AppConfig,
) -> Result<()> {
    info!("Starting audio processing pipeline");

    // Initialize translation components
    let translator = translation::Translator::new(
        config.stt,
        config.translation,
        config.tts,
    )?;
    info!("Translation engine initialized");

    // Create shutdown signal
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = shutdown_signal.clone();
    audio_manager.set_shutdown_signal(shutdown_signal_clone);

    let mut translator = translator; // make mutable

    tokio::select! {
        res = audio_manager.start_processing(translator) => {
            if let Err(e) = res {
                error!("Audio processing failed: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
            shutdown_signal.store(true, Ordering::Relaxed);
            
            // Give some time for graceful shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    audio_manager.stop()?;
    Ok(())
}

fn init_logging(config: &AppConfig, debug_flag: bool) -> Result<()> {
    use tracing_subscriber::fmt;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use std::fs::OpenOptions;
    use std::fs;
    
    // Determine log level
    let log_level = if debug_flag {
        Level::DEBUG
    } else {
        match config.logging.level.as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    };
    
    // Create file layer if configured
    if let Some(log_file) = &config.logging.log_file {
        // Remove existing log file to start fresh
        if fs::metadata(log_file).is_ok() {
            if let Err(e) = fs::remove_file(log_file) {
                eprintln!("Warning: Failed to remove existing log file '{}': {}", log_file, e);
            }
        }
        
        // Test if we can write to the log file
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_file) {
            Ok(mut test_file) => {
                // Test write access
                if let Err(e) = writeln!(test_file, "Log file test write") {
                    eprintln!("Warning: Cannot write to log file '{}': {}", log_file, e);
                    // Fallback to console only
                    tracing_subscriber::fmt()
                        .with_max_level(log_level)
                        .with_ansi(false)
                        .init();
                    
                    error!("Failed to write to log file '{}': {}", log_file, e);
                    info!("Logging to console only");
                } else {
                    // Reset file for actual logging
                    test_file.set_len(0).ok();
                    test_file.seek(std::io::SeekFrom::Start(0)).ok();
                    
                    // Initialize with both console and file output
                    tracing_subscriber::registry()
                        .with(fmt::layer().with_ansi(false))
                        .with(fmt::layer().with_ansi(false).with_writer(test_file))
                        .init();
                    
                    info!("Logging initialized with console and file output: {}", log_file);
                }
            }
            Err(e) => {
                // Fallback to console only if file creation fails
                tracing_subscriber::fmt()
                    .with_max_level(log_level)
                    .with_ansi(false)
                    .init();
                
                error!("Failed to create log file '{}': {}", log_file, e);
                info!("Logging to console only");
            }
        }
    } else {
        // Console only
        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .with_ansi(false)
            .init();
        
        info!("Logging initialized with console output only");
    }
    
    Ok(())
} 