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
    #[arg(short, long, default_value = "en")]
    source_lang: String,
    
    /// Target language for translation
    #[arg(short, long, default_value = "es")]
    target_lang: String,
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
    
    // Load configuration first to get logging settings
    let mut config = AppConfig::load(&args.config)?;
    
    // Override configuration with command line arguments
    config.translation.source_language = args.source_lang.clone();
    config.translation.target_language = args.target_lang.clone();
    
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
    let mut audio_manager = AudioManager::new(config.audio.clone())?;
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