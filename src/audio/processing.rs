use crate::config::AudioConfig;
use crate::error::{Result, VoipGlotError};
use crate::translation::Translator;
use tracing::{info, error, debug, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AudioProcessor {
    config: AudioConfig,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    silence_threshold: f32,
    chunk_duration_ms: u32,
}

impl AudioProcessor {
    pub fn new(config: AudioConfig) -> Result<Self> {
        info!("Initializing AudioProcessor");
        
        Ok(Self {
            config,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            silence_threshold: 0.02, // Will be overridden by config
            chunk_duration_ms: 200,  // Will be overridden by config
        })
    }

    pub fn update_processing_config(&mut self, silence_threshold: f32, chunk_duration_ms: u32) {
        self.silence_threshold = silence_threshold;
        self.chunk_duration_ms = chunk_duration_ms;
        info!("Updated processing config: silence_threshold={}, chunk_duration_ms={}", 
              silence_threshold, chunk_duration_ms);
    }

    pub async fn process_audio(
        &mut self,
        audio_data: Vec<f32>,
        translator: &mut Translator,
    ) -> Result<Option<Vec<f32>>> {
        debug!("Processing audio chunk of {} samples", audio_data.len());
        
        // Check if audio contains speech (not just silence)
        if !self.contains_speech(&audio_data) {
            debug!("Audio chunk contains only silence, skipping");
            return Ok(None);
        }
        
        // Add audio to buffer
        let mut buffer = self.audio_buffer.lock().await;
        buffer.extend(audio_data);
        
        // Check if we have enough audio for processing
        // Increased chunk duration for better stability and reduced CPU usage
        let samples_needed = (self.config.sample_rate as u32 * 400 / 1000) as usize; // 400ms chunks
        
        if buffer.len() < samples_needed {
            // Only log occasionally to avoid spam
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 200 == 0 { // Further reduced logging frequency
                    debug!("Not enough audio samples yet ({} < {})", buffer.len(), samples_needed);
                }
            }
            return Ok(None);
        }
        
        // Extract audio chunk for processing
        let audio_chunk: Vec<f32> = buffer.drain(..samples_needed).collect();
        drop(buffer); // Release lock
        
        // Process the audio through the translation pipeline
        match translator.process_audio_pipeline(audio_chunk).await {
            Ok(translated_audio) => {
                if let Some(audio) = translated_audio {
                    debug!("Successfully processed audio pipeline, generated {} samples", audio.len());
                    Ok(Some(audio))
                } else {
                    debug!("Audio pipeline completed but no output generated");
                    Ok(None)
                }
            }
            Err(e) => {
                error!("Failed to process audio pipeline: {}", e);
                Ok(None)
            }
        }
    }

    fn contains_speech(&self, audio_data: &[f32]) -> bool {
        // Improved energy-based speech detection with better threshold
        let energy: f32 = audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32;
        let rms = energy.sqrt();
        
        // Use a slightly higher threshold to reduce false positives
        let threshold = self.silence_threshold * 1.2;
        debug!("Audio RMS: {}, threshold: {}", rms, threshold);
        rms > threshold
    }

    pub fn set_silence_threshold(&mut self, threshold: f32) {
        self.silence_threshold = threshold;
        info!("Silence threshold set to {}", threshold);
    }

    pub fn set_chunk_duration(&mut self, duration_ms: u32) {
        self.chunk_duration_ms = duration_ms;
        info!("Chunk duration set to {}ms", duration_ms);
    }

    pub fn get_audio_stats(&self) -> AudioStats {
        // This would provide real-time audio statistics
        AudioStats {
            buffer_size: 0, // Would be calculated from actual buffer
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
        }
    }

    pub async fn clear_buffer(&mut self) {
        // Clear the audio buffer
        let mut buffer = self.audio_buffer.lock().await;
        buffer.clear();
        debug!("Audio buffer cleared");
    }
}

#[derive(Debug, Clone)]
pub struct AudioStats {
    pub buffer_size: usize,
    pub sample_rate: u32,
    pub channels: u16,
}

// Audio preprocessing utilities
pub mod preprocessing {
    use crate::error::Result;

    pub fn apply_noise_reduction(audio_data: &[f32]) -> Result<Vec<f32>> {
        // Simple noise reduction using a moving average filter
        let window_size = 5;
        let mut filtered = Vec::with_capacity(audio_data.len());
        
        for i in 0..audio_data.len() {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2 + 1).min(audio_data.len());
            let window = &audio_data[start..end];
            
            let average = window.iter().sum::<f32>() / window.len() as f32;
            filtered.push(average);
        }
        
        Ok(filtered)
    }

    pub fn apply_echo_cancellation(audio_data: &[f32]) -> Result<Vec<f32>> {
        // Placeholder for echo cancellation
        // In a real implementation, this would use more sophisticated algorithms
        Ok(audio_data.to_vec())
    }

    pub fn normalize_audio(audio_data: &[f32]) -> Result<Vec<f32>> {
        let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        
        if max_amplitude > 0.0 {
            let normalized: Vec<f32> = audio_data.iter()
                .map(|&x| x / max_amplitude * 0.8) // Scale to 80% of max
                .collect();
            Ok(normalized)
        } else {
            Ok(audio_data.to_vec())
        }
    }
} 