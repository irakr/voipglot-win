use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
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
        
        let (sender, receiver) = mpsc::channel(500);
        
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
        
        // If a specific device is configured, try to find it
        if let Some(device_name) = &config.output_device {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name.contains(device_name) {
                        info!("Found configured output device: {}", name);
                        return Ok(Some(device));
                    }
                }
            }
            warn!("Configured output device '{}' not found, using default", device_name);
        }
        
        // Use default output device
        let default_device = host.default_output_device();
        if let Some(device) = &default_device {
            if let Ok(name) = device.name() {
                info!("Using default output device: {}", name);
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
                Self::audio_callback(data, &audio_buffer);
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
        
        // Try to use the configured sample rate, fall back to supported config
        let sample_rate = if supported_configs.sample_rate().0 >= self.config.sample_rate {
            cpal::SampleRate(self.config.sample_rate)
        } else {
            supported_configs.sample_rate()
        };
        
        let config = cpal::StreamConfig {
            channels: self.config.channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Fixed(self.config.buffer_size as u32),
        };
        
        info!("Using stream config: {:?}", config);
        Ok(config)
    }

    fn audio_callback(
        data: &mut [f32],
        audio_buffer: &Arc<Mutex<Vec<f32>>>,
    ) {
        let mut buffer = audio_buffer.lock().unwrap();
        
        // Fill the output buffer with available audio data
        for (i, sample) in data.iter_mut().enumerate() {
            if let Some(&audio_sample) = buffer.get(i) {
                *sample = audio_sample;
            } else {
                *sample = 0.0; // Silence if no more data
            }
        }
        
        // Remove the samples we just used
        let samples_used = data.len().min(buffer.len());
        buffer.drain(..samples_used);
        
        // Request more audio data if buffer is getting low
        if buffer.len() < 2048 {
            debug!("Audio buffer running low ({} samples), requesting more data", buffer.len());
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

    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        // Volume control would be implemented here
        // This is a placeholder for future implementation
        info!("Setting playback volume to {}", volume);
        Ok(())
    }
} 