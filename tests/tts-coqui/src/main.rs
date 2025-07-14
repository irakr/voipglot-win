use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use coqui_tts::Synthesizer;
use log::{debug, error, info, warn};
use serde::Deserialize;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    audio: AudioConfig,
    tts: TtsConfig,
    logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
struct AudioConfig {
    input_device: String,
    output_device: String,
    sample_rate: u32,
    channels: u16,
    buffer_size: usize,
}

#[derive(Debug, Deserialize)]
struct TtsConfig {
    model_path: String,
    voice_speed: f32,
    voice_pitch: f32,
    enable_gpu: bool,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
}

fn load_config() -> Result<Config> {
    let config_content = fs::read_to_string("config.toml")
        .context("Failed to read config.toml")?;
    
    let config: Config = toml::from_str(&config_content)
        .context("Failed to parse config.toml")?;
    
    Ok(config)
}

fn find_device_by_name(host: &cpal::Host, device_name: &str) -> Result<Option<cpal::Device>> {
    if device_name.is_empty() {
        return Ok(None);
    }
    
    let devices: Vec<_> = host.output_devices()?.collect();
    for device in devices {
        if let Ok(name) = device.name() {
            if name == device_name {
                return Ok(Some(device));
            }
        }
    }
    
    Err(anyhow::anyhow!("Device '{}' not found", device_name))
}

fn find_compatible_config(device: &cpal::Device, target_sample_rate: u32) -> Result<StreamConfig> {
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
    
    // Try to find a config that exactly matches our target sample rate
    for config in &supported_configs {
        if config.max_sample_rate().0 == target_sample_rate {
            let channels = config.channels();
            
            debug!(
                "Using exact match configuration: {}Hz, {} channels, {:?}",
                target_sample_rate, channels, config.sample_format()
            );
            
            return Ok(StreamConfig {
                channels,
                sample_rate: cpal::SampleRate(target_sample_rate),
                buffer_size: cpal::BufferSize::Default,
            });
        }
    }
    
    // If exact match not found, try to find a config that supports the target rate
    for config in &supported_configs {
        if config.max_sample_rate().0 >= target_sample_rate {
            let channels = config.channels();
            
            debug!(
                "Using compatible configuration: {}Hz, {} channels, {:?}",
                target_sample_rate, channels, config.sample_format()
            );
            
            return Ok(StreamConfig {
                channels,
                sample_rate: cpal::SampleRate(target_sample_rate),
                buffer_size: cpal::BufferSize::Default,
            });
        }
    }
    
    // If exact match not found, find the closest supported rate
    let mut best_config = None;
    let mut best_diff = u32::MAX;
    
    for config in &supported_configs {
        let supported_rate = config.max_sample_rate().0;
        let diff = if supported_rate >= target_sample_rate {
            supported_rate - target_sample_rate
        } else {
            target_sample_rate - supported_rate
        };
        
        if diff < best_diff {
            best_diff = diff;
            best_config = Some(config);
        }
    }
    
    if let Some(config) = best_config {
        let sample_rate = config.max_sample_rate().0;
        let channels = config.channels();
        
        warn!(
            "Target sample rate {}Hz not supported, using closest supported rate: {}Hz",
            target_sample_rate, sample_rate
        );
        
        return Ok(StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        });
    }
    
    // Fallback to first available config
    if let Some(config) = supported_configs.first() {
        let sample_rate = config.max_sample_rate().0;
        let channels = config.channels();
        
        warn!(
            "Target sample rate {}Hz not supported, using {}Hz",
            target_sample_rate, sample_rate
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

fn resample_audio(samples: Vec<f32>, from_rate: u32, to_rate: u32, channels: u16) -> Vec<f32> {
    if from_rate == to_rate {
        return samples;
    }
    
    let ratio = to_rate as f64 / from_rate as f64;
    let samples_per_channel = samples.len() / channels as usize;
    let new_samples_per_channel = (samples_per_channel as f64 * ratio) as usize;
    let new_length = new_samples_per_channel * channels as usize;
    let mut resampled = Vec::with_capacity(new_length);
    
    // Ultra-efficient resampling using linear interpolation
    let mut src_pos = 0.0;
    let step = 1.0 / ratio;
    
    for _ in 0..new_samples_per_channel {
        let src_index = src_pos as usize;
        let frac = src_pos - src_index as f64;
        
        if src_index < samples_per_channel - 1 {
            // Linear interpolation between two samples
            for ch in 0..channels as usize {
                let idx1 = src_index * channels as usize + ch;
                let idx2 = (src_index + 1) * channels as usize + ch;
                let sample1 = samples[idx1];
                let sample2 = samples[idx2];
                let interpolated = sample1 + (sample2 - sample1) * frac as f32;
                resampled.push(interpolated);
            }
        } else if src_index < samples_per_channel {
            // Last sample, no interpolation needed
            for ch in 0..channels as usize {
                let idx = src_index * channels as usize + ch;
                resampled.push(samples[idx]);
            }
        } else {
            // Beyond source, pad with zeros
            for _ in 0..channels as usize {
                resampled.push(0.0);
            }
        }
        
        src_pos += step;
    }
    
    resampled
}

fn play_audio_buffer(
    device: cpal::Device,
    config: StreamConfig,
    audio_buffer: Vec<f32>,
) -> Result<()> {
    let device_sample_rate = config.sample_rate.0;
    let channels = config.channels;
    let tts_sample_rate = 22050; // TTS output sample rate
    
    info!("Playing audio: {} samples, TTS: {}Hz, Device: {}Hz, {} channels", 
          audio_buffer.len(), tts_sample_rate, device_sample_rate, channels);
    
    // Convert f32 samples to the device's format and ensure they're in valid range
    let mut samples: Vec<f32> = audio_buffer.into_iter()
        .map(|sample| sample.clamp(-1.0, 1.0))
        .collect();
    
    // Convert mono TTS output to stereo if needed
    if channels == 2 && samples.len() > 0 {
        let mono_samples = samples;
        samples = Vec::with_capacity(mono_samples.len() * 2);
        for sample in mono_samples {
            samples.push(sample); // Left channel
            samples.push(sample); // Right channel (same as left for mono source)
        }
        info!("Converted mono TTS output to stereo: {} samples", samples.len());
    }
    
    // Only resample if absolutely necessary and the difference is significant
    if device_sample_rate != tts_sample_rate && (device_sample_rate as f64 / tts_sample_rate as f64).abs() > 1.1 {
        info!("Resampling from {}Hz to {}Hz", tts_sample_rate, device_sample_rate);
        let resample_start = std::time::Instant::now();
        samples = resample_audio(samples, tts_sample_rate, device_sample_rate, channels);
        let resample_time = resample_start.elapsed();
        info!("Resampling completed in {:?}", resample_time);
    } else if device_sample_rate != tts_sample_rate {
        info!("Sample rate difference is small, skipping resampling ({}Hz -> {}Hz)", tts_sample_rate, device_sample_rate);
    }
    
    // Create a shared buffer for streaming
    let samples_arc = Arc::new(Mutex::new(samples));
    let position_arc = Arc::new(Mutex::new(0usize));
    let finished_arc = Arc::new(Mutex::new(false));
    
    // Create a stream to play the audio
    let stream = device.build_output_stream(
        &config,
        {
            let samples_arc = Arc::clone(&samples_arc);
            let position_arc = Arc::clone(&position_arc);
            let finished_arc = Arc::clone(&finished_arc);
            
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let samples_guard = samples_arc.lock().unwrap();
                let mut position_guard = position_arc.lock().unwrap();
                let mut finished_guard = finished_arc.lock().unwrap();
                
                let samples = &*samples_guard;
                let position = &mut *position_guard;
                
                for i in 0..data.len() {
                    if *position < samples.len() {
                        data[i] = samples[*position];
                        *position += 1;
                    } else {
                        data[i] = 0.0; // Silence after audio ends
                        *finished_guard = true;
                    }
                }
            }
        },
        |err| error!("Audio stream error: {}", err),
        None,
    )?;
    
    stream.play()?;
    
    // Wait for audio to finish using a non-blocking approach
    let start_time = std::time::Instant::now();
    let expected_duration = Duration::from_secs_f64(samples_arc.lock().unwrap().len() as f64 / device_sample_rate as f64);
    let timeout = expected_duration + Duration::from_millis(500); // Add 500ms buffer
    
    while start_time.elapsed() < timeout {
        if *finished_arc.lock().unwrap() {
            break;
        }
        thread::sleep(Duration::from_millis(10)); // Small sleep to prevent busy waiting
    }
    
    stream.pause()?;
    Ok(())
}

fn main() -> Result<()> {
    // Check if user wants to just list devices
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--list-devices" {
        return list_devices();
    }
    
    // Load configuration
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load config.toml: {}. Using default configuration.", e);
            Config {
                audio: AudioConfig {
                    input_device: String::new(),
                    output_device: String::new(),
                    sample_rate: 48000,
                    channels: 2,
                    buffer_size: 2048,
                },
                tts: TtsConfig {
                    model_path: String::new(),
                    voice_speed: 1.0,
                    voice_pitch: 1.0,
                    enable_gpu: false,
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                },
            }
        }
    };
    
    // Initialize logging
    std::env::set_var("RUST_LOG", &config.logging.level);
    env_logger::init();
    
    info!("Starting Coqui TTS test application");
    info!("Press Ctrl+C to exit");
    info!("Use --list-devices to see available audio devices");
    
    // Initialize Coqui TTS synthesizer
    let model_name = if !config.tts.model_path.is_empty() {
        &config.tts.model_path
    } else {
        "tts_models/en/ljspeech/tacotron2-DDC"  // Default model
    };
    
    let mut synthesizer = Synthesizer::new(model_name, config.tts.enable_gpu);
    
    // Configure synthesizer for better performance
    if config.tts.voice_speed != 1.0 {
        info!("Setting voice speed to: {}", config.tts.voice_speed);
        // Note: Coqui TTS may not support speed adjustment in this version
    }
    
    info!("Coqui TTS synthesizer initialized successfully");
    
    // Get audio output device
    let host = cpal::default_host();
    let device = if !config.audio.output_device.is_empty() {
        find_device_by_name(&host, &config.audio.output_device)?
            .context("Specified output device not found")?
    } else {
        host.default_output_device()
            .context("No default output device found")?
    };
    
    info!("Using output device: {}", device.name()?);
    
    // Find compatible audio configuration - try to match TTS sample rate first
    debug!("Attempting to find compatible audio configuration...");
    let tts_sample_rate = 22050; // TTS output sample rate
    
    // First try to use device's preferred sample rate
    let stream_config = match find_compatible_config(&device, config.audio.sample_rate) {
        Ok(config) => {
            info!("Using device's preferred sample rate: {}Hz", config.sample_rate.0);
            config
        },
        Err(_) => {
            // Fall back to TTS sample rate as last resort
            info!("Device's preferred rate not supported, trying TTS rate: {}Hz", tts_sample_rate);
            find_compatible_config(&device, tts_sample_rate)?
        }
    };
    
    info!(
        "Selected audio config: {}Hz, {} channels, buffer size: {:?}",
        stream_config.sample_rate.0, stream_config.channels, stream_config.buffer_size
    );
    
    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    println!("\nEnter text to speak (press Enter after each line, Ctrl+C to exit):");
    while running.load(Ordering::SeqCst) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        print!("[{}] > ", timestamp);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let text = input.trim();
        if text.is_empty() {
            continue;
        }
        
        // Optimize text processing for better performance
        let text_to_speak = {
            // Remove excessive whitespace and normalize text
            let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
            
            // Limit text length to prevent performance issues
            if normalized.len() > 150 {  // Reduced from 200 to 150 for better performance
                warn!("Text too long ({} chars), truncating to 150 characters for better performance", normalized.len());
                normalized[..150].to_string()
            } else {
                normalized
            }
        };
        
        info!("Converting text to speech: {}", text_to_speak);
        
        // Synthesize speech and get audio buffer
        let start_time = std::time::Instant::now();
        let audio_buffer = {
            // Process in a separate scope to ensure memory is freed immediately
            let buffer = synthesizer.tts(&text_to_speak);
            
            if buffer.is_empty() {
                error!("TTS synthesis returned empty audio buffer");
                continue;
            }
            
            // Normalize audio levels for consistent volume
            let max_amplitude = buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
            if max_amplitude > 0.0 {
                buffer.into_iter().map(|x| (x / max_amplitude) * 0.95).collect()
            } else {
                buffer
            }
        };
        
        let synthesis_time = start_time.elapsed();
        info!("Speech synthesis completed in {:?}, got {} samples", synthesis_time, audio_buffer.len());
        
        // Play the audio buffer (coqui-tts returns f32 samples)
        if let Err(e) = play_audio_buffer(device.clone(), stream_config.clone(), audio_buffer) {
            error!("Failed to play audio: {}", e);
        }
    }
    
    info!("Application shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        let config = load_config();
        assert!(config.is_ok(), "Failed to load configuration");
    }
    
    #[test]
    fn test_synthesizer_initialization() {
        let _synthesizer = Synthesizer::new("tts_models/en/ljspeech/tacotron2-DDC", false);
        // Synthesizer::new doesn't return Result, so we just check it doesn't panic
    }
} 