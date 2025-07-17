use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat, Stream,
};
use crate::config::AudioConfig;
use crate::error::{Result, VoipGlotError};
use tracing::{info, error, debug, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AudioPlayback {
    config: AudioConfig,
    host: cpal::Host,
    device: Option<cpal::Device>,
    stream: Option<Stream>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sender: Option<mpsc::Sender<Vec<f32>>>,
    receiver: Option<mpsc::Receiver<Vec<f32>>>,
}

impl AudioPlayback {
    pub fn new(config: AudioConfig) -> Result<Self> {
        info!("Initializing AudioPlayback");
        
        let host = cpal::default_host();
        let device = Self::find_output_device(&host, &config)?;
        
        let (sender, receiver) = mpsc::channel(100);
        
        Ok(Self {
            config,
            host,
            device,
            stream: None,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            sender: Some(sender),
            receiver: Some(receiver),
        })
    }

    fn find_output_device(host: &cpal::Host, config: &AudioConfig) -> Result<Option<cpal::Device>> {
        let devices = host.output_devices()
            .map_err(|e| VoipGlotError::Audio(format!("Failed to enumerate output devices: {}", e)))?;
        
        info!("Available output devices:");
        let mut device_list = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                device_list.push((device, name.clone()));
                info!("  - {}", name);
            }
        }
        
        // If a specific device is configured, try to find it
        if let Some(device_name) = &config.output_device {
            for (device, name) in &device_list {
                if name.contains(device_name) {
                    info!("Found configured output device: {}", name);
                    return Ok(Some(device.clone()));
                }
            }
            warn!("Configured output device '{}' not found", device_name);
        }
        
        // If output_device is None (empty string in config), use system default
        // Only use VB Cable as fallback if output_device was explicitly set but not found
        if config.output_device.is_none() {
            // Use default output device when no specific device is configured
            let default_device = host.default_output_device();
            if let Some(device) = &default_device {
                if let Ok(name) = device.name() {
                    info!("Using default output device: {}", name);
                }
            }
            return Ok(default_device);
        }
        
        // Only try VB Cable as fallback if output_device was specified but not found
        if !config.vb_cable_device.is_empty() {
            for (device, name) in &device_list {
                if name.contains(&config.vb_cable_device) {
                    info!("Falling back to VB Cable device: {}", name);
                    return Ok(Some(device.clone()));
                }
            }
            warn!("Configured VB Cable device '{}' not found", config.vb_cable_device);
        }
        
        // Final fallback to default output device
        let default_device = host.default_output_device();
        if let Some(device) = &default_device {
            if let Ok(name) = device.name() {
                info!("Using default output device as final fallback: {}", name);
            }
        }
        
        Ok(default_device)
    }

    pub fn start(&mut self) -> Result<()> {
        let device = self.device.as_ref()
            .ok_or_else(|| VoipGlotError::DeviceNotFound("No output device available".to_string()))?;
        
        info!("Starting audio playback to device");
        
        let config = self.build_stream_config(device)?;
        let audio_buffer = Arc::clone(&self.audio_buffer);
        let channels = config.channels as usize;
        
        // Start a background task to receive audio data and add it to the buffer
        let receiver = self.receiver.take()
            .ok_or_else(|| VoipGlotError::Audio("Receiver already taken".to_string()))?;
        
        let buffer_clone = Arc::clone(&audio_buffer);
        tokio::spawn(async move {
            Self::audio_receiver_task(receiver, buffer_clone).await;
        });
        
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                Self::audio_callback(data, &audio_buffer, channels);
            },
            |err| {
                error!("Audio playback error: {}", err);
            },
            None,
        )?;
        
        stream.play()?;
        self.stream = Some(stream);
        
        info!("Audio playback started successfully");
        Ok(())
    }

    async fn audio_receiver_task(
        mut receiver: mpsc::Receiver<Vec<f32>>,
        audio_buffer: Arc<Mutex<Vec<f32>>>,
    ) {
        while let Some(audio_data) = receiver.recv().await {
            let mut buffer = audio_buffer.lock().unwrap();
            let len = audio_data.len();
            buffer.extend(audio_data);
            debug!("Added {} samples to playback buffer", len);
        }
        debug!("Audio receiver task ended");
    }

    fn build_stream_config(&self, device: &cpal::Device) -> Result<cpal::StreamConfig> {
        let supported_configs = device.default_output_config()
            .map_err(|e| VoipGlotError::Audio(format!("Failed to get default output config: {}", e)))?;
        
        info!("Supported output config: {:?}", supported_configs);
        
        // Use the device's supported configuration and handle conversion in the callback
        let config = cpal::StreamConfig {
            channels: supported_configs.channels(),
            sample_rate: supported_configs.sample_rate(),
            buffer_size: cpal::BufferSize::Fixed(self.config.buffer_size as u32),
        };
        
        info!("Using stream config: {:?}", config);
        Ok(config)
    }

    fn audio_callback(
        data: &mut [f32],
        audio_buffer: &Arc<Mutex<Vec<f32>>>,
        channels: usize,
    ) {
        let mut buffer = audio_buffer.lock().unwrap();
        
        // Fill the output buffer with available audio data
        // Handle multi-channel output by duplicating mono samples
        let samples_per_frame = channels;
        
        for frame in data.chunks_mut(samples_per_frame) {
            if let Some(&audio_sample) = buffer.get(0) {
                // Duplicate mono sample to all channels
                for sample in frame.iter_mut() {
                    *sample = audio_sample;
                }
                // Remove the sample we just used
                buffer.drain(0..1);
            } else {
                // Silence if no more data
                for sample in frame.iter_mut() {
                    *sample = 0.0;
                }
            }
        }
        
        // Request more audio data if buffer is getting low
        if buffer.len() < 1024 {
            // Only log occasionally to avoid spam
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 100 == 0 {
                    debug!("Audio buffer running low ({} samples), requesting more data", buffer.len());
                }
            }
        }
    }

    pub async fn write_audio_chunk(&mut self, audio_data: Vec<f32>) -> Result<()> {
        let sender = self.sender.as_ref()
            .ok_or_else(|| VoipGlotError::Audio("Sender not available".to_string()))?;
        
        sender.send(audio_data).await
            .map_err(|e| VoipGlotError::Audio(format!("Failed to send audio chunk: {}", e)))?;
        
        debug!("Audio chunk queued for playback");
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping audio playback");
        
        if let Some(stream) = &self.stream {
            stream.pause()?;
        }
        
        self.stream = None;
        info!("Audio playback stopped");
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<String>> {
        let devices = self.host.output_devices()
            .map_err(|e| VoipGlotError::Audio(format!("Failed to enumerate devices: {}", e)))?;
        
        let mut device_names = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                device_names.push(name);
            }
        }
        
        Ok(device_names)
    }

    pub fn find_vb_cable_device(&self) -> Result<Option<cpal::Device>> {
        let devices = self.host.output_devices()
            .map_err(|e| VoipGlotError::Audio(format!("Failed to enumerate devices: {}", e)))?;
        
        for device in devices {
            if let Ok(name) = device.name() {
                if name.contains("CABLE Input") || name.contains("VB-Audio") {
                    info!("Found VB Cable device: {}", name);
                    return Ok(Some(device));
                }
            }
        }
        
        warn!("VB Cable device not found");
        Ok(None)
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        // Volume control would be implemented here
        // This is a placeholder for future implementation
        info!("Setting playback volume to {}", volume);
        Ok(())
    }
} 