use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig, Device, Host,
};
use coqui_tts::Synthesizer;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::config::AppConfig;

pub struct TTSProcessor {
    text_rx: mpsc::UnboundedReceiver<String>,
    config: AppConfig,
    synthesizer: Option<Synthesizer>,
    audio_device: Option<Device>,
    stream_config: Option<StreamConfig>,
}

impl TTSProcessor {
    pub fn new(config: AppConfig, text_rx: mpsc::UnboundedReceiver<String>) -> Result<Self> {
        info!("Initializing Coqui TTS processor");
        
        // Initialize Coqui TTS synthesizer
        let model_path = &config.tts.model_path;
        info!("Loading TTS model: {}", model_path);
        
        let synthesizer = Synthesizer::new(model_path, config.tts.enable_gpu);
        
        info!("Coqui TTS synthesizer initialized successfully");
        
        // Initialize audio output device and configuration
        let (audio_device, stream_config) = Self::setup_audio_output(&config)?;
        
        Ok(Self {
            text_rx,
            config,
            synthesizer: Some(synthesizer),
            audio_device: Some(audio_device),
            stream_config: Some(stream_config),
        })
    }
    
    fn setup_audio_output(config: &AppConfig) -> Result<(Device, StreamConfig)> {
        info!("Setting up audio output device");
        
        let host = cpal::default_host();
        let device = Self::find_output_device(&host, &config.audio_output.output_device)?;
        
        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Using output device: {}", device_name);
        
        // Find compatible configuration - cross-check with device capabilities
        let stream_config = Self::find_compatible_config(&device, config)?;
        
        info!(
            "Selected audio config: {}Hz, {} channels, buffer size: {:?}",
            stream_config.sample_rate.0, stream_config.channels, stream_config.buffer_size
        );
        
        Ok((device, stream_config))
    }
    
    fn find_output_device(host: &Host, device_name: &Option<String>) -> Result<Device> {
        match device_name {
            Some(name) if !name.trim().is_empty() => {
                // Find specific device by name
                let devices: Vec<_> = host.output_devices()?.collect();
                for device in devices {
                    if let Ok(dev_name) = device.name() {
                        if dev_name == *name {
                            info!("Found specified output device: {}", name);
                            return Ok(device);
                        }
                    }
                }
                return Err(anyhow::anyhow!("Specified output device '{}' not found", name));
            }
            _ => {
                // Use default device
                info!("Using default output device");
                host.default_output_device()
                    .context("No default output device found")
            }
        }
    }
    
    fn find_compatible_config(device: &Device, config: &AppConfig) -> Result<StreamConfig> {
        let supported_configs: Vec<_> = device.supported_output_configs()?.collect();
        let target_sample_rate = config.audio_output.sample_rate;
        let target_channels = config.audio_output.channels;
        
        info!("Device supported configurations:");
        for config in &supported_configs {
            info!(
                "  Sample rate: {}Hz, Channels: {}, Format: {:?}",
                config.max_sample_rate().0,
                config.channels(),
                config.sample_format()
            );
        }
        
        // Validate user configuration against device capabilities
        let mut found_valid_config = false;
        let mut selected_sample_rate = target_sample_rate;
        let mut selected_channels = target_channels;
        
        // Try to find exact match first
        for config_range in &supported_configs {
            if config_range.max_sample_rate().0 >= target_sample_rate &&
               config_range.channels() == target_channels {
                found_valid_config = true;
                break;
            }
        }
        
        if !found_valid_config {
            warn!("User-configured audio settings not supported by device");
            
            // Find best alternative - prioritize sample rate over channels
            let mut best_config = None;
            let mut best_score = f64::INFINITY;
            
            for config_range in &supported_configs {
                let rate_diff = (config_range.max_sample_rate().0 as f64 - target_sample_rate as f64).abs();
                let channel_diff = (config_range.channels() as f64 - target_channels as f64).abs();
                let score = rate_diff / target_sample_rate as f64 + channel_diff / target_channels as f64;
                
                if score < best_score {
                    best_score = score;
                    best_config = Some(config_range);
                }
            }
            
            if let Some(best) = best_config {
                selected_sample_rate = best.max_sample_rate().0;
                selected_channels = best.channels();
                
                warn!(
                    "Auto-adjusting audio config from {}Hz/{}ch to {}Hz/{}ch (device capabilities)",
                    target_sample_rate, target_channels,
                    selected_sample_rate, selected_channels
                );
            } else {
                return Err(anyhow::anyhow!("No compatible audio configuration found"));
            }
        } else {
            info!("User audio configuration is compatible with device");
        }
        
        Ok(StreamConfig {
            channels: selected_channels,
            sample_rate: cpal::SampleRate(selected_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        })
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
        
        // Linear interpolation resampling
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
    
    pub fn synthesize_speech(&mut self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let synthesizer = self.synthesizer.as_mut()
            .context("TTS synthesizer not initialized")?;
        
        debug!("Synthesizing speech for text: \"{}\"", text);
        
        // Optimize text for better performance
        let text_to_speak = {
            let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if normalized.len() > 150 {
                warn!("Text too long ({} chars), truncating to 150 characters for better performance", normalized.len());
                normalized[..150].to_string()
            } else {
                normalized
            }
        };
        
        let start_time = std::time::Instant::now();
        let audio_buffer = synthesizer.tts(&text_to_speak);
        let synthesis_time = start_time.elapsed();
        
        if audio_buffer.is_empty() {
            error!("TTS synthesis returned empty audio buffer");
            return Ok(Vec::new());
        }
        
        info!("Speech synthesis completed in {:?}, got {} samples", synthesis_time, audio_buffer.len());
        
        // Normalize audio levels for consistent volume
        let max_amplitude = audio_buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let normalized_buffer = if max_amplitude > 0.0 {
            audio_buffer.into_iter().map(|x| (x / max_amplitude) * 0.95).collect()
        } else {
            audio_buffer
        };
        
        Ok(normalized_buffer)
    }
    
    fn play_audio_buffer(&self, audio_buffer: Vec<f32>) -> Result<()> {
        let device = self.audio_device.as_ref().context("Audio device not initialized")?;
        let config = self.stream_config.as_ref().context("Stream config not initialized")?;
        
        let device_sample_rate = config.sample_rate.0;
        let channels = config.channels;
        let tts_sample_rate = 22050; // Coqui TTS output sample rate
        
        info!("Playing audio: {} samples, TTS: {}Hz, Device: {}Hz, {} channels", 
              audio_buffer.len(), tts_sample_rate, device_sample_rate, channels);
        
        // Ensure samples are in valid range
        let mut samples: Vec<f32> = audio_buffer.into_iter()
            .map(|sample| sample.clamp(-1.0, 1.0))
            .collect();
        
        // Convert mono TTS output to stereo if needed
        if channels == 2 && !samples.is_empty() {
            let mono_samples = samples;
            samples = Vec::with_capacity(mono_samples.len() * 2);
            for sample in mono_samples {
                samples.push(sample); // Left channel
                samples.push(sample); // Right channel (same as left for mono source)
            }
            info!("Converted mono TTS output to stereo: {} samples", samples.len());
        }
        
        // Resample if necessary
        if device_sample_rate != tts_sample_rate {
            info!("Resampling from {}Hz to {}Hz", tts_sample_rate, device_sample_rate);
            let resample_start = std::time::Instant::now();
            samples = Self::resample_audio(samples, tts_sample_rate, device_sample_rate, channels);
            let resample_time = resample_start.elapsed();
            info!("Resampling completed in {:?}", resample_time);
        }
        
        // Create shared buffer for streaming
        let samples_arc = Arc::new(Mutex::new(samples));
        let position_arc = Arc::new(Mutex::new(0usize));
        let finished_arc = Arc::new(Mutex::new(false));
        
        // Create output stream
        let stream = device.build_output_stream(
            config,
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
        
        // Wait for audio to finish
        let start_time = std::time::Instant::now();
        let expected_duration = Duration::from_secs_f64(samples_arc.lock().unwrap().len() as f64 / device_sample_rate as f64);
        let timeout = expected_duration + Duration::from_millis(500);
        
        while start_time.elapsed() < timeout {
            if *finished_arc.lock().unwrap() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        stream.pause()?;
        Ok(())
    }
    
    pub async fn start_processing(&mut self) -> Result<()> {
        info!("Starting TTS processing loop");
        
        while let Some(text) = self.text_rx.recv().await {
            if text.trim().is_empty() {
                continue;
            }
            
            info!("Processing TTS for: \"{}\"", text);
            
            match self.synthesize_speech(&text) {
                Ok(audio_buffer) => {
                    if !audio_buffer.is_empty() {
                        if let Err(e) = self.play_audio_buffer(audio_buffer) {
                            error!("Failed to play audio: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("TTS synthesis failed: {}", e);
                }
            }
        }
        
        info!("TTS processing loop ended");
        Ok(())
    }
}
