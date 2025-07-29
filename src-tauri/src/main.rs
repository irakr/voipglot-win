//! # VoipGlot Windows Application
//! 
//! This is the Windows-specific application that uses the VoipGlot Core library
//! to provide real-time audio translation for Windows gaming and VOIP applications.
//! 
//! This application uses Tauri to provide a modern GUI interface.

mod lib;

use lib::create_app;

fn main() {
    // Initialize logging
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }

    // Create and run Tauri app
    let app = create_app();
    app.run(|_app_handle, _event| {});
}

fn init_logging() -> anyhow::Result<()> {
    use tracing_subscriber::fmt;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;
    use std::fs;
    
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
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_ansi(false))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .init();
    
    tracing::info!("VoipGlot Windows Tauri application started");
    tracing::info!("Logging initialized with console and file output: {}", log_file);
    
    Ok(())
} 