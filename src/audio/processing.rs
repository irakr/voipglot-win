use crate::config::{AudioConfig, ProcessingConfig};
use crate::error::Result;
use crate::translation::Translator;
use tracing::{info, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AudioProcessor {
    config: AudioConfig,
    processing_config: ProcessingConfig,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    silence_threshold: f32,
    chunk_duration_ms: u32,
    passthrough_mode: bool,
}

impl AudioProcessor {
    pub fn new(audio_config: AudioConfig, processing_config: ProcessingConfig) -> Result<Self> {
        info!("Initializing AudioProcessor");
        
        Ok(Self {
            config: audio_config,
            processing_config: processing_config.clone(),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            silence_threshold: processing_config.silence_threshold,
            chunk_duration_ms: processing_config.chunk_duration_ms,
            passthrough_mode: false,
        })
    }

    pub async fn process_audio(
        &mut self,
        audio_data: Vec<f32>,
        translator: &Translator,
    ) -> Result<Option<Vec<f32>>> {
        debug!("Processing audio chunk of {} samples", audio_data.len());
        
        // AUDIO PASSTHROUGH MODE: Forward microphone directly to output
        if self.passthrough_mode {
            debug!("Passthrough mode: forwarding audio directly");
            return Ok(Some(audio_data));
        }
        
        // Check if audio contains speech (not just silence)
        if !self.contains_speech(&audio_data) {
            debug!("Audio chunk contains only silence, skipping");
            // Return silence audio to maintain buffer
            return Ok(Some(vec![0.0; audio_data.len()]));
        }
        
        // Add audio to buffer
        let mut buffer = self.audio_buffer.lock().await;
        buffer.extend(audio_data.clone());
        
        // Check if we have enough audio for processing
        let samples_needed = (self.config.sample_rate as u32 * self.chunk_duration_ms / 1000) as usize;
        
        // Limit buffer size to prevent overflow
        let max_buffer_size = samples_needed * 4; // Keep max 4 chunks in buffer
        if buffer.len() > max_buffer_size {
            debug!("Buffer overflow detected ({} > {}), truncating", buffer.len(), max_buffer_size);
            let excess = buffer.len() - max_buffer_size;
            buffer.drain(..excess);
        }
        
        if buffer.len() < samples_needed {
            debug!("Not enough audio samples yet ({} < {})", buffer.len(), samples_needed);
            // Return silence to maintain buffer while collecting more audio
            return Ok(Some(vec![0.0; audio_data.len()]));
        }
        
        // Extract audio chunk for processing
        let audio_chunk: Vec<f32> = buffer.drain(..samples_needed).collect();
        drop(buffer); // Release lock
        
        // Process the audio through the translation pipeline
        match self.translate_audio(audio_chunk, translator).await {
            Ok(translated_audio) => {
                debug!("Successfully translated audio");
                if translated_audio.is_empty() {
                    // Return silence if no translation was generated
                    Ok(Some(vec![0.0; audio_data.len()]))
                } else {
                    // Ensure the translated audio has the right length
                    let mut result = translated_audio;
                    if result.len() != audio_data.len() {
                        // Pad or truncate to match input length
                        if result.len() < audio_data.len() {
                            result.extend(vec![0.0; audio_data.len() - result.len()]);
                        } else {
                            result.truncate(audio_data.len());
                        }
                    }
                    Ok(Some(result))
                }
            }
            Err(e) => {
                error!("Failed to translate audio: {}", e);
                // Return silence on error to maintain buffer
                Ok(Some(vec![0.0; audio_data.len()]))
            }
        }
    }

    fn contains_speech(&self, audio_data: &[f32]) -> bool {
        // Simple energy-based speech detection
        let energy: f32 = audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32;
        let rms = energy.sqrt();
        
        debug!("Audio RMS: {}, threshold: {}", rms, self.silence_threshold);
        rms > self.silence_threshold
    }

    async fn translate_audio(
        &self,
        audio_data: Vec<f32>,
        translator: &Translator,
    ) -> Result<Vec<f32>> {
        info!("Starting audio translation pipeline");
        println!("🎯 STARTING TRANSLATION PIPELINE with {} samples", audio_data.len());
        
        // Step 1: Speech-to-Text
        let text = translator.speech_to_text(audio_data).await?;
        
        // Check if we have actual speech transcription
        if text.trim().is_empty() {
            debug!("STT returned empty string, no speech detected or transcription failed");
            println!("❌ STT returned empty string - no speech detected");
            // Return empty audio since no speech was transcribed
            return Ok(Vec::new());
        } else {
            // Normal flow with actual speech transcription
            info!("STT Result: '{}'", text);
            
            // Step 2: Translation
            let translated_text = translator.translate_text(&text).await?;
            info!("Translation: '{}' -> '{}'", text, translated_text);
            
            // Step 3: Text-to-Speech
            let translated_audio = translator.text_to_speech(&translated_text).await?;
            info!("TTS completed, generated {} samples", translated_audio.len());
            println!("✅ TTS completed: {} samples generated", translated_audio.len());
            
            Ok(translated_audio)
        }
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

    pub fn enable_passthrough_mode(&mut self) {
        self.passthrough_mode = true;
        info!("Audio passthrough mode ENABLED - microphone audio will be forwarded directly to output");
    }

    pub fn disable_passthrough_mode(&mut self) {
        self.passthrough_mode = false;
        info!("Audio passthrough mode DISABLED - AI processing enabled");
    }

    pub fn is_passthrough_mode(&self) -> bool {
        self.passthrough_mode
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