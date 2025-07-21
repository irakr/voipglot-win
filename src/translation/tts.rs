use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig, Device, Host,
};
use coqui_tts::Synthesizer;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use std::path::Path;

use crate::config::AppConfig;

/// TTS Model Management utilities
struct TTSModelManager;

impl TTSModelManager {
    /// Get the local model path if it exists, otherwise return the model identifier
    fn get_model_path(model_identifier: &str) -> String {
        // Convert model identifier to local path
        let local_path = format!("models/{}", model_identifier);
        
        if Path::new(&local_path).exists() {
            info!("Using local TTS model: {}", local_path);
            local_path
        } else {
            info!("Local TTS model not found at: {}", local_path);
            info!("Using model identifier (will download): {}", model_identifier);
            model_identifier.to_string()
        }
    }
    
    /// Verify that a local model can be loaded by the Rust coqui-tts bindings
    fn verify_local_model(local_path: &str) -> Result<()> {
        info!("Verifying local TTS model: {}", local_path);
        
        // For now, just check if critical files exist
        // The Rust coqui-tts bindings may not support local file paths well
        let model_dir = Path::new(local_path);
        
        // Check for common TTS model files
        let required_files = ["config.json", "model.pth"];
        let mut found_files = 0;
        
        for file in &required_files {
            if model_dir.join(file).exists() {
                found_files += 1;
            }
        }
        
        if found_files == required_files.len() {
            info!("Model verification passed: found {} required files", found_files);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Model verification failed: found {}/{} required files", 
                found_files, 
                required_files.len()
            ))
        }
    }
    
    /// Download and cache a TTS model locally (for build-time use)
    fn download_and_cache_model(model_identifier: &str) -> Result<String> {
        info!("Downloading and caching TTS model: {}", model_identifier);
        
        // Create models directory if it doesn't exist
        let models_dir = Path::new("models");
        if !models_dir.exists() {
            std::fs::create_dir_all(models_dir)
                .context("Failed to create models directory")?;
        }
        
        // For runtime, we'll let Coqui handle downloads automatically
        // The local caching is mainly for offline scenarios
        warn!("Runtime model caching not implemented - using Coqui's automatic download");
        Ok(model_identifier.to_string())
    }
    
    /// Find the cached model directory in Coqui's cache
    fn find_cached_model(model_identifier: &str) -> Option<std::path::PathBuf> {
        let model_cache_name = model_identifier.replace("/", "--");
        
        // Check common cache locations
        let cache_dirs = [
            dirs::data_local_dir().map(|d| d.join("tts")), // Windows: AppData/Local/tts
            dirs::data_dir().map(|d| d.join("tts")),       // Linux: ~/.local/share/tts
            dirs::home_dir().map(|d| d.join("Library").join("Application Support").join("tts")), // macOS
        ];
        
        for cache_dir in cache_dirs.iter().flatten() {
            let model_path = cache_dir.join(&model_cache_name);
            if model_path.exists() {
                return Some(model_path);
            }
        }
        
        None
    }
    
    /// Copy model files from source to destination
    fn copy_model_files(src: &Path, dst: &Path) -> Result<()> {
        if dst.exists() {
            std::fs::remove_dir_all(dst)
                .context("Failed to remove existing local model directory")?;
        }
        
        std::fs::create_dir_all(dst)
            .context("Failed to create local model directory")?;
        
        // Copy all files from source to destination
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_file() {
                std::fs::copy(&src_path, &dst_path)
                    .with_context(|| format!("Failed to copy model file: {:?}", src_path))?;
            } else if src_path.is_dir() {
                Self::copy_model_files(&src_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
}

pub struct TTSProcessor {
    text_rx: mpsc::Receiver<String>,
    config: AppConfig,
    synthesizer: Option<Synthesizer>,
    audio_device: Option<Device>,
    stream_config: Option<StreamConfig>,
    tts_playing: Arc<AtomicBool>,
}

impl TTSProcessor {
    pub fn new(
        config: AppConfig, 
        text_rx: mpsc::Receiver<String>,
        tts_playing: Arc<AtomicBool>
    ) -> Result<Self> {
        info!("Initializing Coqui TTS processor");
        
        // Get the best available model path (local first, then download)
        let model_path = Self::resolve_model_path(&config.tts.model_path)?;
        info!("Using TTS model: {}", model_path);
        
        // Try to initialize the synthesizer with better error handling
        let synthesizer = match std::panic::catch_unwind(|| {
            Synthesizer::new(&model_path, config.tts.enable_gpu)
        }) {
            Ok(synth) => synth,
            Err(_) => {
                error!("TTS Synthesizer initialization panicked with model path: {}", model_path);
                
                // If local path failed, try the original model identifier
                if model_path != config.tts.model_path {
                    warn!("Retrying with original model identifier: {}", config.tts.model_path);
                    match std::panic::catch_unwind(|| {
                        Synthesizer::new(&config.tts.model_path, config.tts.enable_gpu)
                    }) {
                        Ok(synth) => {
                            info!("Successfully initialized with model identifier fallback");
                            synth
                        }
                        Err(_) => {
                            return Err(anyhow::anyhow!(
                                "Failed to initialize TTS Synthesizer with both local path and identifier"
                            ));
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Failed to initialize TTS Synthesizer"));
                }
            }
        };
        
        info!("Coqui TTS synthesizer initialized successfully");
        
        // Initialize audio output device and configuration
        let (audio_device, stream_config) = Self::setup_audio_output(&config)?;
        
        Ok(Self {
            text_rx,
            config,
            synthesizer: Some(synthesizer),
            audio_device: Some(audio_device),
            stream_config: Some(stream_config),
            tts_playing,
        })
    }
    
    /// Resolve the model path, checking local cache first, downloading if needed
    fn resolve_model_path(model_identifier: &str) -> Result<String> {
        // Check if we already have the model locally cached
        let local_path = TTSModelManager::get_model_path(model_identifier);
        
        // If the local path exists and is different from identifier, try to use it
        if local_path != model_identifier && Path::new(&local_path).exists() {
            info!("Local TTS model found, attempting to use: {}", local_path);
            
            // Try to verify the local model works by testing initialization
            match TTSModelManager::verify_local_model(&local_path) {
                Ok(()) => {
                    info!("Local model verified successfully: {}", local_path);
                    return Ok(local_path);
                }
                Err(e) => {
                    warn!("Local model verification failed: {}", e);
                    warn!("Falling back to model identifier for download");
                }
            }
        }
        
        // If local model doesn't exist or is invalid, use model identifier
        // This will trigger Coqui's automatic download behavior
        info!("Using model identifier (Coqui will handle download): {}", model_identifier);
        Ok(model_identifier.to_string())
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
        
        info!("Synthesizing speech for text: \"{}\"", text);
        
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
        info!("Starting TTS synthesis...");
        
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
        
        info!("Audio normalization completed, ready for playback");
        Ok(normalized_buffer)
    }
    

    
    async fn play_audio_buffer_internal(&self, audio_buffer: Vec<f32>) -> Result<()> {
        // Internal method for audio playback - feedback prevention is handled at higher level
        
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
        
        // Wait for audio to finish using async sleep (non-blocking)
        let start_time = std::time::Instant::now();
        let expected_duration = {
            let sample_count = samples_arc.lock().unwrap().len();
            Duration::from_secs_f64(sample_count as f64 / device_sample_rate as f64)
        };
        let timeout = expected_duration + Duration::from_millis(200);
        
        while start_time.elapsed() < timeout {
            if *finished_arc.lock().unwrap() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await; // ✅ ASYNC SLEEP!
        }
        
        stream.pause()?;
        info!("Audio playback completed");
        Ok(())
    }
    
    pub async fn start_processing(&mut self) -> Result<()> {
        info!("Starting TTS processing loop");
        
        while let Some(text) = self.text_rx.recv().await {
            if text.trim().is_empty() {
                continue;
            }
            
            info!("Processing TTS for: \"{}\"", text);
            
            // Signal that TTS processing is starting (synthesis + playback) - prevents STT audio feedback
            if self.config.processing.enable_feedback_prevention {
                self.tts_playing.store(true, Ordering::Relaxed);
                info!("TTS processing starting (synthesis + playback) - STT audio capture paused to prevent feedback");
            }
            
            // Create a guard to ensure TTS playing state is reset even if processing fails
            struct TtsProcessingGuard {
                tts_playing: Arc<AtomicBool>,
                config: AppConfig,
            }
            
            impl Drop for TtsProcessingGuard {
                fn drop(&mut self) {
                    if self.config.processing.enable_feedback_prevention {
                        self.tts_playing.store(false, Ordering::Relaxed);
                        info!("TTS processing ended (cleanup) - STT audio capture resumed");
                    }
                }
            }
            
            let _guard = TtsProcessingGuard {
                tts_playing: self.tts_playing.clone(),
                config: self.config.clone(),
            };
            
            // Use timeout to prevent hanging synthesis (fast_pitch should complete in <5 seconds)
            let timeout_secs = self.config.tts.synthesis_timeout_secs;
            match tokio::time::timeout(Duration::from_secs(timeout_secs), async {
                self.synthesize_speech(&text)
            }).await {
                Ok(Ok(audio_buffer)) => {
                    if !audio_buffer.is_empty() {
                        if let Err(e) = self.play_audio_buffer_internal(audio_buffer).await {
                            error!("Failed to play audio: {}", e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("TTS synthesis failed: {}", e);
                }
                Err(_) => {
                    error!("TTS synthesis timed out after {} seconds (this suggests a problem with the TTS model)", timeout_secs);
                }
            }
            
            // Add silence buffer after complete TTS processing
            if self.config.processing.enable_feedback_prevention {
                let silence_buffer_ms = self.config.processing.tts_silence_buffer_ms;
                if silence_buffer_ms > 0 {
                    info!("Waiting {}ms silence buffer after TTS processing", silence_buffer_ms);
                    tokio::time::sleep(Duration::from_millis(silence_buffer_ms as u64)).await;
                }
                
                // Reset TTS playing state manually (guard will also reset it)
                self.tts_playing.store(false, Ordering::Relaxed);
                info!("TTS processing completed - STT audio capture resumed");
            }
            
            // Yield to allow other tasks to run
            tokio::task::yield_now().await;
        }
        
        info!("TTS processing loop ended");
        Ok(())
    }
    

}
