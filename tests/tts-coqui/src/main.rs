use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    StreamConfig,
};
use log::{debug, error, info};
use std::io::{self, Write};
use tts::Tts;

fn find_compatible_config(device: &cpal::Device) -> Result<StreamConfig> {
    let supported_configs: Vec<_> = device.supported_output_configs()?.collect();
    
    info!("Device supported configurations:");
    for config in &supported_configs {
        info!(
            "  Sample rate: {}Hz, Channels: {}, Format: {:?}",
            config.max_sample_rate().0,
            config.channels(),
            config.sample_format()
        );
    }
    
    if let Some(config) = supported_configs.first() {
        let sample_rate = config.max_sample_rate().0;
        let channels = config.channels();
        
        debug!(
            "Using device's native configuration: {}Hz, {} channels, {:?}",
            sample_rate, channels, config.sample_format()
        );
        
        return Ok(StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        });
    }
    
    Err(anyhow::anyhow!("No supported audio configuration found"))
}

fn list_devices() -> Result<()> {
    let host = cpal::default_host();
    let devices: Vec<_> = host.output_devices()?.collect();
    
    println!("Available output devices:");
    for (i, device) in devices.iter().enumerate() {
        if let Ok(name) = device.name() {
            println!("  {}: {}", i, name);
            
            if let Ok(configs) = device.supported_output_configs() {
                for config in configs {
                    println!(
                        "    - {}Hz, {} channels, {:?}",
                        config.max_sample_rate().0,
                        config.channels(),
                        config.sample_format()
                    );
                }
            }
        }
    }
    
    if let Some(default_device) = host.default_output_device() {
        if let Ok(name) = default_device.name() {
            println!("\nDefault output device: {}", name);
            if let Some(device_index) = devices.iter().position(|d| d.name().unwrap_or_default() == name)
            {
                println!("Default device index: {} (0-based)", device_index);
            }
        }
    } else {
        println!("\nNo default output device found");
    }
    
    Ok(())
}

fn main() -> Result<()> {
    // Check if user wants to just list devices
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--list-devices" {
        return list_devices();
    }
    
    // Initialize logging
    std::env::set_var("RUST_LOG", "debug,cpal=debug");
    env_logger::init();
    
    info!("Starting Coqui TTS test application");
    info!("Press Ctrl+C to exit");
    info!("Use --list-devices to see available audio devices");
    
    // Initialize TTS engine
    let mut tts = Tts::default()
        .context("Failed to initialize TTS engine")?;
    
    // List available voices
    let voices = tts.voices();
    println!("\nAvailable voices:");
    for voice in voices {
        println!("  - {:?}", voice);
    }
    
    // Get audio output device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("No output device found")?;
    
    info!("Using output device: {}", device.name()?);
    
    // Find compatible audio configuration
    debug!("Attempting to find compatible audio configuration...");
    let config = find_compatible_config(&device)?;
    
    info!(
        "Selected audio config: {}Hz, {} channels, buffer size: {:?}",
        config.sample_rate.0, config.channels, config.buffer_size
    );
    
    println!("\nEnter text to speak (press Enter after each line, Ctrl+C to exit):");
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let text = input.trim();
        if text.is_empty() {
            continue;
        }
        
        info!("Converting text to speech: {}", text);
        
        // Synthesize speech
        match tts.speak(text, true) {
            Ok(_) => debug!("Speech synthesis completed successfully"),
            Err(e) => error!("Failed to synthesize speech: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tts_initialization() {
        let tts = Tts::default();
        assert!(tts.is_ok(), "Failed to initialize TTS engine");
    }
} 