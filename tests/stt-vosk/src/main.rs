use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use dasp::Sample;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use log::{error, info, warn, debug};
use vosk::{Model, Recognizer, DecodingState};

/// Extract the transcribed text from a VOSK result string
fn extract_text_from_result(result: &str) -> Option<String> {
    debug!("Processing VOSK result: {}", result);
    
    // For CompleteResultSingle format
    if result.contains("CompleteResultSingle") {
        if let Some(text_start) = result.find("text: \"") {
            if let Some(text_end) = result[text_start..].find("\"}") {
                let text = result[text_start + 7..text_start + text_end].to_string();
                debug!("Extracted text from CompleteResultSingle: {}", text);
                return Some(text);
            }
        }
    }
    
    // For other result formats, try to extract any text field
    if let Some(text_start) = result.find("\"text\":\"") {
        if let Some(text_end) = result[text_start + 8..].find('\"') {
            let text = result[text_start + 8..text_start + 8 + text_end].to_string();
            debug!("Extracted text from JSON format: {}", text);
            return Some(text);
        }
    }
    
    debug!("No text could be extracted from result");
    None
}

fn find_compatible_config(device: &cpal::Device) -> Result<StreamConfig> {
    // Get supported configs and collect into vector for multiple iterations
    let supported_configs: Vec<_> = device.supported_input_configs()?.collect();
    
    info!("Device supported configurations:");
    for config in &supported_configs {
        info!("  Sample rate: {}Hz, Channels: {}, Format: {:?}", 
              config.max_sample_rate().0, config.channels(), config.sample_format());
    }
    
    // Instead of trying to force 16kHz, use the device's native configuration
    if let Some(config) = supported_configs.first() {
        let sample_rate = config.max_sample_rate().0;
        let channels = config.channels();
        
        debug!("Using device's native configuration: {}Hz, {} channels, {:?}", 
               sample_rate, channels, config.sample_format());
        
        // Create a configuration that matches the device's capabilities
        return Ok(StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(1024),
        });
    }
    
    Err(anyhow::anyhow!("No supported audio configuration found"))
}

fn list_devices() -> Result<()> {
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices()?.collect();
    
    println!("Available input devices:");
    for (i, device) in devices.iter().enumerate() {
        if let Ok(name) = device.name() {
            println!("  {}: {}", i, name);
            
            // Show supported configs for each device
            if let Ok(configs) = device.supported_input_configs() {
                for config in configs {
                    println!("    - {}Hz, {} channels, {:?}", 
                            config.max_sample_rate().0, config.channels(), config.sample_format());
                }
            }
        }
    }
    
    if let Some(default_device) = host.default_input_device() {
        if let Ok(name) = default_device.name() {
            println!("\nDefault input device: {}", name);
            if let Some(device_index) = devices.iter().position(|d| d.name().unwrap_or_default() == name) {
                println!("Default device index: {} (0-based)", device_index);
            }
            
            // Show default device capabilities
            if let Ok(configs) = default_device.supported_input_configs() {
                println!("Default device capabilities:");
                for config in configs {
                    println!("  - {}Hz, {} channels, {:?}", 
                            config.max_sample_rate().0, config.channels(), config.sample_format());
                }
            }
        }
    } else {
        println!("\nNo default input device found");
    }
    
    Ok(())
}

fn main() -> Result<()> {
    // Check if user wants to just list devices
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--list-devices" {
        return list_devices();
    }
    
    // Initialize logging with more verbose output
    std::env::set_var("RUST_LOG", "debug,cpal=debug");
    env_logger::init();
    
    info!("Starting VOSK STT test application");
    info!("Press Ctrl+C to exit");
    info!("Use --list-devices to see available audio devices");
    
    // List all available input devices
    let host = cpal::default_host();
    debug!("Audio host backend: {:?}", host.id());
    let devices: Vec<_> = host.input_devices()?.collect();
    
    info!("Available input devices:");
    for (i, device) in devices.iter().enumerate() {
        if let Ok(name) = device.name() {
            info!("  {}: {}", i, name);
            if let Ok(configs) = device.supported_input_configs() {
                for config in configs {
                    debug!("    Supported config: {}Hz-{}Hz, {} channels, {:?}", 
                           config.min_sample_rate().0,
                           config.max_sample_rate().0,
                           config.channels(),
                           config.sample_format());
                }
            }
        }
    }
    
    // Get audio host and device
    let device = host
        .default_input_device()
        .context("No input device found")?;
    
    info!("Using input device: {}", device.name()?);
    
    // Show device index for reference
    if let Some(device_index) = devices.iter().position(|d| d.name().unwrap_or_default() == device.name().unwrap_or_default()) {
        info!("Device index: {} (0-based)", device_index);
    }
    
    // Find compatible audio configuration
    debug!("Attempting to find compatible audio configuration...");
    let config = find_compatible_config(&device)?;
    
    info!("Selected audio config: {}Hz, {} channels, buffer size: {:?}", 
          config.sample_rate.0, config.channels, config.buffer_size);
    
    // Initialize VOSK model
    let model_path = std::env::var("VOSK_MODEL_PATH")
        .unwrap_or_else(|_| "models/vosk-model-small-en-us-0.15".to_string());
    
    info!("Loading VOSK model from: {}", model_path);
    debug!("Checking if model path exists: {}", std::path::Path::new(&model_path).exists());
    
    let model = Model::new(&model_path)
        .ok_or_else(|| {
            error!("Failed to load VOSK model. Path exists: {}", std::path::Path::new(&model_path).exists());
            anyhow::anyhow!("Failed to load VOSK model from: {}. Please ensure VOSK_MODEL_PATH points to a valid model directory", model_path)
        })?;
    
    debug!("VOSK model loaded successfully");
    
    // Create VOSK recognizer with the device's sample rate
    info!("Creating VOSK recognizer with sample rate: {}Hz", config.sample_rate.0);
    let recognizer = Recognizer::new(&model, config.sample_rate.0 as f32)
        .ok_or_else(|| anyhow::anyhow!("Failed to create VOSK recognizer"))?;
    
    debug!("VOSK recognizer created successfully");
    let recognizer = Arc::new(Mutex::new(recognizer));
    
    // Create channels for communication between audio thread and main thread
    let (tx, rx) = mpsc::channel::<String>();
    
    debug!("Setting up audio stream...");
    // Audio callback function
    let recognizer_clone = recognizer.clone();
    let audio_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // Convert stereo to mono if needed
        let mono_samples: Vec<f32> = if config.channels == 2 {
            // Convert stereo to mono by averaging channels
            data.chunks(2).map(|chunk| {
                if chunk.len() == 2 {
                    (chunk[0] + chunk[1]) / 2.0
                } else {
                    chunk[0]
                }
            }).collect()
        } else {
            data.to_vec()
        };
        
        // Convert f32 samples to i16 for VOSK
        let samples: Vec<i16> = mono_samples
            .iter()
            .map(|&sample| {
                // Ensure the sample is in the [-1.0, 1.0] range before converting
                let clamped = sample.max(-1.0).min(1.0);
                (clamped * 32767.0) as i16
            })
            .collect();
        
        // Process audio with VOSK
        if let Ok(mut rec) = recognizer_clone.lock() {
            let state = rec.accept_waveform(&samples);
            
            match state {
                DecodingState::Finalized => {
                    let result = rec.result();
                    debug!("Got final result from VOSK: {:?}", result);
                    let result_str = format!("{:?}", result);
                    if !result_str.is_empty() && result_str != "\"\"" {
                        let _ = tx.send(result_str);
                    }
                },
                DecodingState::Running => {
                    let partial = rec.partial_result();
                    let partial_str = format!("{:?}", partial);
                    if !partial_str.is_empty() && partial_str != "\"\"" && !partial_str.contains("partial: \"\"") {
                        debug!("Got partial result: {}", partial_str);
                        let _ = tx.send(format!("[Partial] {}", partial_str));
                    }
                },
                _ => {}
            }
        }
    };
    
    // Build and start audio stream
    debug!("Building audio input stream with config: {:?}", config);
    let stream = device.build_input_stream(
        &config,
        audio_callback,
        |err| error!("Audio stream error: {}", err),
        None,
    ).context("Failed to build audio input stream")?;
    
    debug!("Starting audio stream...");
    stream.play().context("Failed to start audio stream")?;
    info!("Audio stream started successfully");
    
    // Main loop to receive transcription results
    info!("Waiting for speech input...");
    for result in rx {
        if result.starts_with("[Partial]") {
            debug!("Received partial result: {}", result);
        } else {
            if let Some(text) = extract_text_from_result(&result) {
                if !text.trim().is_empty() {
                    println!("\nTranscription: {}", text);
                }
            } else {
                debug!("Could not extract text from result: {}", result);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vosk_model_loading() {
        // This test would require a VOSK model to be present
        // For now, just test that the code compiles
        assert!(true);
    }
} 