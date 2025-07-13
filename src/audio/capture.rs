use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat, Stream,
};
use crate::config::AudioConfig;
use crate::error::{Result, VoipGlotError};
use tracing::{info, error, debug, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AudioCapture {
    config: AudioConfig,
    host: cpal::Host,
    device: Option<cpal::Device>,
    stream: Option<Stream>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sender: Option<mpsc::Sender<Vec<f32>>>,
    receiver: Option<mpsc::Receiver<Vec<f32>>>,
}

impl AudioCapture {
    pub fn new(config: AudioConfig) -> Result<Self> {
        info!("Initializing AudioCapture");
        
        let host = cpal::default_host();
        let device = Self::find_input_device(&host, &config)?;
        
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

    fn find_input_device(host: &cpal::Host, config: &AudioConfig) -> Result<Option<cpal::Device>> {
        let devices = host.input_devices()
            .map_err(|e| VoipGlotError::Audio(format!("Failed to enumerate input devices: {}", e)))?;
        
        info!("Available input devices:");
        let mut device_list = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                device_list.push((device, name.clone()));
                info!("  - {}", name);
            }
        }
        
        // If a specific device is configured, try to find it
        if let Some(device_name) = &config.input_device {
            for (device, name) in &device_list {
                if name.contains(device_name) {
                    info!("Found configured input device: {}", name);
                    return Ok(Some(device.clone()));
                }
            }
            warn!("Configured input device '{}' not found", device_name);
        }
        
        // Use default input device
        let default_device = host.default_input_device();
        if let Some(device) = &default_device {
            if let Ok(name) = device.name() {
                info!("Using default input device: {}", name);
            }
        }
        
        Ok(default_device)
    }

    pub fn start(&mut self) -> Result<()> {
        let device = self.device.as_ref()
            .ok_or_else(|| VoipGlotError::DeviceNotFound("No input device available".to_string()))?;
        
        info!("Starting audio capture from device");
        
        let config = self.build_stream_config(device)?;
        let audio_buffer = Arc::clone(&self.audio_buffer);
        let sender = self.sender.as_ref()
            .ok_or_else(|| VoipGlotError::Audio("Sender not available".to_string()))?;
        
        let device_sample_rate = config.sample_rate.0;
        let sender_clone = sender.clone();
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                Self::audio_callback(data, &audio_buffer, &sender_clone, device_sample_rate);
            },
            |err| {
                error!("Audio capture error: {}", err);
            },
            None,
        )?;
        
        stream.play()?;
        self.stream = Some(stream);
        
        info!("Audio capture started successfully");
        Ok(())
    }

    fn build_stream_config(&self, device: &cpal::Device) -> Result<cpal::StreamConfig> {
        let supported_configs = device.default_input_config()
            .map_err(|e| VoipGlotError::Audio(format!("Failed to get default input config: {}", e)))?;
        
        info!("Supported input config: {:?}", supported_configs);
        
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
        data: &[f32],
        audio_buffer: &Arc<Mutex<Vec<f32>>>,
        sender: &mpsc::Sender<Vec<f32>>,
        device_sample_rate: u32,
    ) {
        // Convert stereo to mono if needed
        let mono_samples: Vec<f32> = if data.len() > 1 && data.len() % 2 == 0 {
            // Assume stereo input, convert to mono
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
        
        // Copy audio data to our buffer
        let mut buffer = audio_buffer.lock().unwrap();
        buffer.extend_from_slice(&mono_samples);
        
        // Calculate chunk size based on target sample rate (16kHz for VOSK)
        // We want 1 second of audio at 16kHz, so we need to accumulate enough samples
        let target_sample_rate: u32 = 16000;
        let samples_per_second = target_sample_rate as usize;
        let chunk_size = samples_per_second;
        
        if buffer.len() >= chunk_size {
            let chunk: Vec<f32> = buffer.drain(..chunk_size).collect();
            
            // Simple sample rate conversion: downsample if needed
            let converted_chunk = if device_sample_rate != target_sample_rate {
                let ratio = device_sample_rate as f32 / target_sample_rate as f32;
                let new_length = (chunk.len() as f32 / ratio) as usize;
                let mut converted = Vec::with_capacity(new_length);
                
                for i in 0..new_length {
                    let src_index = (i as f32 * ratio) as usize;
                    if src_index < chunk.len() {
                        converted.push(chunk[src_index]);
                    }
                }
                converted
            } else {
                chunk
            };
            
            // Try to send the chunk, but don't block if the receiver is slow
            if let Err(e) = sender.try_send(converted_chunk) {
                debug!("Failed to send audio chunk: {}", e);
            }
        }
    }

    pub async fn read_audio_chunk(&mut self) -> Result<Vec<f32>> {
        let receiver = self.receiver.as_mut()
            .ok_or_else(|| VoipGlotError::Audio("Receiver not available".to_string()))?;
        
        match receiver.recv().await {
            Some(chunk) => {
                debug!("Received audio chunk of {} samples", chunk.len());
                Ok(chunk)
            }
            None => {
                Err(VoipGlotError::Audio("Audio capture channel closed".to_string()))
            }
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping audio capture");
        
        if let Some(stream) = &self.stream {
            stream.pause()?;
        }
        
        self.stream = None;
        info!("Audio capture stopped");
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<String>> {
        let devices = self.host.input_devices()
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
        let devices = self.host.input_devices()
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
} 