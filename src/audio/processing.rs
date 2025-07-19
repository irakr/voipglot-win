use anyhow::Result;
use tracing::{debug, info};

use crate::config::AppConfig;

pub struct AudioProcessor {
    config: AppConfig,
    running: bool,
}

impl AudioProcessor {
    pub fn new(config: AppConfig) -> Self {
        info!("Initializing Audio Processor");
        
        Self {
            config,
            running: false,
        }
    }
    
    pub fn start_processing(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        info!("Starting audio processing");
        self.running = true;
        
        Ok(())
    }
    
    pub fn stop_processing(&mut self) {
        if !self.running {
            return;
        }
        
        info!("Stopping audio processing");
        self.running = false;
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn process_audio_chunk(&self, audio_data: &[i16]) -> Result<Vec<i16>> {
        // Apply audio preprocessing
        let processed = self.apply_preprocessing(audio_data)?;
        
        // Check for silence
        if self.is_silence(&processed) {
            debug!("Silence detected, skipping processing");
            return Ok(Vec::new());
        }
        
        Ok(processed)
    }
    
    fn apply_preprocessing(&self, audio_data: &[i16]) -> Result<Vec<i16>> {
        let mut processed = audio_data.to_vec();
        
        // Apply noise reduction if configured
        if self.config.processing.noise_reduction {
            self.apply_noise_reduction(&mut processed);
        }
        
        // Apply normalization if configured
        if self.config.processing.echo_cancellation {
            self.apply_normalization(&mut processed);
        }
        
        Ok(processed)
    }
    
    fn apply_noise_reduction(&self, audio_data: &mut [i16]) {
        // Simple noise gate implementation
        let threshold = self.config.processing.silence_threshold * i16::MAX as f32;
        
        for sample in audio_data.iter_mut() {
            if sample.abs() < threshold as i16 {
                *sample = 0;
            }
        }
    }
    
    fn apply_normalization(&self, audio_data: &mut [i16]) {
        if audio_data.is_empty() {
            return;
        }
        
        // Find the maximum absolute value
        let max_val = audio_data.iter().map(|&x| x.abs()).max().unwrap();
        
        if max_val > 0 {
            // Normalize to 80% of max range
            let scale_factor = (i16::MAX as f32 * 0.8) / max_val as f32;
            
            for sample in audio_data.iter_mut() {
                *sample = (*sample as f32 * scale_factor) as i16;
            }
        }
    }
    
    fn is_silence(&self, audio_data: &[i16]) -> bool {
        if audio_data.is_empty() {
            return true;
        }
        
        // Calculate RMS (Root Mean Square) for silence detection
        let sum_squares: f64 = audio_data.iter().map(|&x| (x as f64).powi(2)).sum();
        let rms = (sum_squares / audio_data.len() as f64).sqrt();
        
        let threshold = self.config.processing.silence_threshold as f64 * i16::MAX as f64;
        
        rms < threshold
    }
}
