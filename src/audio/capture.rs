use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use dasp::Sample;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::AppConfig;

pub struct AudioCapture {
    config: AppConfig,
    audio_tx: Option<mpsc::UnboundedSender<Vec<i16>>>,
    stream: Option<cpal::Stream>,
}

impl AudioCapture {
    pub fn new(config: AppConfig) -> Self {
        info!("Initializing Audio Capture");
        
        Self {
            config,
            audio_tx: None,
            stream: None,
        }
    }
    
    pub fn start_capture(&mut self, audio_tx: mpsc::UnboundedSender<Vec<i16>>) -> Result<()> {
        info!("Starting audio capture");
        
        // Get default input device
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
        
        info!("Using input device: {}", device.name().unwrap_or_default());
        
        // Get device config
        let config = device
            .default_input_config()
            .map_err(|e| anyhow::anyhow!("Failed to get input config: {}", e))?;
        
        let channels = config.channels();
        let sample_rate = config.sample_rate().0;
        
        info!("Audio config: {}Hz, {} channels, {:?} format", 
              sample_rate, channels, config.sample_format());
        
        self.audio_tx = Some(audio_tx);
        
        // Build audio stream
        let err_fn = |err| {
            error!("Audio stream error: {}", err);
        };
        
        let audio_tx = self.audio_tx.as_ref().unwrap().clone();
        let stream = match config.sample_format() {
            SampleFormat::I8 => device.build_input_stream(
                &config.into(),
                move |data: &[i8], _| {
                    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
                    let data = if channels != 1 {
                        Self::stereo_to_mono(&data)
                    } else {
                        data
                    };
                    
                    if let Err(e) = audio_tx.send(data) {
                        error!("Failed to send audio data: {}", e);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let data = if channels != 1 {
                        Self::stereo_to_mono(data)
                    } else {
                        data.to_vec()
                    };
                    
                    if let Err(e) = audio_tx.send(data) {
                        error!("Failed to send audio data: {}", e);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I32 => device.build_input_stream(
                &config.into(),
                move |data: &[i32], _| {
                    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
                    let data = if channels != 1 {
                        Self::stereo_to_mono(&data)
                    } else {
                        data
                    };
                    
                    if let Err(e) = audio_tx.send(data) {
                        error!("Failed to send audio data: {}", e);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
                    let data = if channels != 1 {
                        Self::stereo_to_mono(&data)
                    } else {
                        data
                    };
                    
                    if let Err(e) = audio_tx.send(data) {
                        error!("Failed to send audio data: {}", e);
                    }
                },
                err_fn,
                None,
            ),
            sample_format => {
                return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format));
            }
        }
        .map_err(|e| anyhow::anyhow!("Failed to build input stream: {}", e))?;
        
        // Start the stream
        stream.play().map_err(|e| anyhow::anyhow!("Failed to play stream: {}", e))?;
        
        self.stream = Some(stream);
        
        info!("Audio capture started successfully");
        Ok(())
    }
    
    pub fn stop_capture(&mut self) {
        info!("Stopping audio capture");
        
        self.stream = None;
        self.audio_tx = None;
        
        info!("Audio capture stopped");
    }
    
    fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
        let mut result = Vec::with_capacity(input_data.len() / 2);
        result.extend(
            input_data
                .chunks_exact(2)
                .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
        );
        result
    }
}
