use crate::error::Result;
use tracing::{info, debug, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

pub struct SpeechToText {
    language: String,
    model_loaded: bool,
    model_path: String,
    whisper_context: Arc<Mutex<Option<WhisperContext>>>,
}

impl SpeechToText {
    pub fn new(language: String) -> Result<Self> {
        info!("Initializing Speech-to-Text with language: {} (Whisper only)", language);
        
        // Determine model path
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                format!("{}/.voipglot/whisper/ggml-base.bin", home)
            });
        
        Ok(Self {
            language,
            model_loaded: false,
            model_path,
            whisper_context: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn transcribe(&self, audio_data: Vec<f32>) -> Result<String> {
        debug!("Transcribing audio with {} samples", audio_data.len());
        
        if audio_data.is_empty() {
            return Ok(String::new());
        }

        // Check if model is loaded
        if !self.model_loaded {
            match self.load_whisper_model().await {
                Ok(_) => {
                    info!("Whisper model loaded successfully");
                }
                Err(e) => {
                    debug!("Whisper model not available: {}, using fallback", e);
                    return self.fallback_transcription(&audio_data);
                }
            }
        }

        // Use Whisper for transcription
        match self.whisper_transcribe(&audio_data).await {
            Ok(transcription) => {
                if !transcription.trim().is_empty() {
                    info!("Whisper transcription: '{}'", transcription);
                    Ok(transcription)
                } else {
                    // Fall back to energy detection if Whisper returns empty
                    self.fallback_transcription(&audio_data)
                }
            }
            Err(e) => {
                warn!("Whisper transcription failed: {}, using fallback", e);
                self.fallback_transcription(&audio_data)
            }
        }
    }

    async fn load_whisper_model(&self) -> Result<()> {
        let mut ctx_guard = self.whisper_context.lock().await;
        if ctx_guard.is_some() {
            return Ok(());
        }

        // Check model availability
        self.check_model_availability()?;
        
        info!("Loading Whisper model from: {}", self.model_path);
        
        // Load the model (this is CPU-intensive, so we do it in a blocking task)
        let model_path = self.model_path.clone();
        let ctx = tokio::task::spawn_blocking(move || {
            WhisperContext::new_with_params(&model_path, Default::default())
        }).await
        .map_err(|e| crate::error::VoipGlotError::Configuration(
            format!("Failed to spawn model loading task: {}", e)
        ))?
        .map_err(|e| crate::error::VoipGlotError::Configuration(
            format!("Failed to load Whisper model: {}", e)
        ))?;
        
        *ctx_guard = Some(ctx);
        info!("Whisper model loaded successfully");
        
        Ok(())
    }

    async fn whisper_transcribe(&self, audio_data: &[f32]) -> Result<String> {
        // Ensure model is loaded
        if !self.model_loaded {
            return Err(crate::error::VoipGlotError::Configuration(
                "Whisper model not loaded. Please ensure model is pre-loaded before transcription.".to_string()
            ));
        }
        
        // Preprocess audio
        let processed_audio = self.preprocess_audio(audio_data.to_vec());
        
        // Convert to 16-bit PCM (Whisper requirement)
        let pcm_data: Vec<i16> = processed_audio
            .iter()
            .map(|&x| (x * 32767.0) as i16)
            .collect();
        // Convert to f32 for Whisper API
        let pcm_data_f32: Vec<f32> = pcm_data.iter().map(|&x| x as f32 / 32767.0).collect();
        
        // Extract language code before spawning task
        let language_code = self.get_whisper_language_code().to_string();
        
        // Verify the context is available (should be pre-loaded)
        {
            let ctx_guard = self.whisper_context.lock().await;
            if ctx_guard.is_none() {
                return Err(crate::error::VoipGlotError::Configuration(
                    "Whisper context not initialized. Please ensure model is pre-loaded.".to_string()
                ));
            }
        }
        
        // Clone everything needed for the blocking task
        let pcm_data_f32_clone = pcm_data_f32.clone();
        let language_code_clone = language_code.clone();
        
        // Since WhisperContext doesn't implement Clone, we need to create a new one
        // But since the model is already loaded, this should be fast
        let model_path = self.model_path.clone();
        
        // Run inference in blocking task to avoid blocking async runtime
        let transcription = tokio::task::spawn_blocking(move || {
            // Create a new context for this task (model is already loaded, so this is fast)
            let ctx = WhisperContext::new_with_params(&model_path, Default::default())
                .map_err(|e| crate::error::VoipGlotError::Configuration(
                    format!("Failed to load Whisper model: {}", e)
                ))?;
            
            let mut state = ctx.create_state()
                .map_err(|e| crate::error::VoipGlotError::Api(
                    format!("Failed to create Whisper state: {}", e)
                ))?;
            
            // Configure params inside the blocking task
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(Some(&language_code_clone));
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_timestamps(false);
            params.set_single_segment(true);
            
            state.full(params, &pcm_data_f32_clone)
                .map_err(|e| crate::error::VoipGlotError::Api(
                    format!("Whisper inference failed: {}", e)
                ))?;
            
            // Get results
            let num_segments = state.full_n_segments()
                .map_err(|e| crate::error::VoipGlotError::Api(
                    format!("Failed to get segment count: {}", e)
                ))?;
            
            let mut transcription = String::new();
            for i in 0..num_segments {
                let segment = state.full_get_segment_text(i)
                    .map_err(|e| crate::error::VoipGlotError::Api(
                        format!("Failed to get segment {}: {}", i, e)
                    ))?;
                transcription.push_str(&segment);
                transcription.push(' ');
            }
            
            Ok::<String, crate::error::VoipGlotError>(transcription.trim().to_string())
        }).await
        .map_err(|e| crate::error::VoipGlotError::Configuration(
            format!("Failed to spawn inference task: {}", e)
        ))?;
        
        transcription
    }

    pub async fn download_and_load_model(&mut self) -> Result<()> {
        // First, download the model if it doesn't exist
        if !std::path::Path::new(&self.model_path).exists() {
            info!("Whisper model not found, downloading...");
            
            // Create directory
            let model_dir = std::path::Path::new(&self.model_path).parent()
                .ok_or_else(|| crate::error::VoipGlotError::Configuration(
                    "Invalid model path".to_string()
                ))?;
            
            std::fs::create_dir_all(model_dir)
                .map_err(|e| crate::error::VoipGlotError::Io(e))?;
            
            // Download model with progress indication
            let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";
            info!("Downloading Whisper model from: {}", url);
            info!("Model size: ~1GB - This may take a few minutes depending on your internet connection...");
            
            // Download with progress tracking
            let response = reqwest::get(url).await
                .map_err(|e| crate::error::VoipGlotError::Api(
                    format!("Failed to download model: {}", e)
                ))?;
            
            let content_length = response.content_length();
            let mut downloaded: u64 = 0;
            let mut stream = response.bytes_stream();
            
            let mut file = tokio::fs::File::create(&self.model_path).await
                .map_err(|e| crate::error::VoipGlotError::Io(e))?;
            
            use tokio::io::AsyncWriteExt;
            use futures::StreamExt;
            
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| crate::error::VoipGlotError::Api(
                    format!("Failed to read model data: {}", e)
                ))?;
                
                file.write_all(&chunk).await
                    .map_err(|e| crate::error::VoipGlotError::Io(e))?;
                
                downloaded += chunk.len() as u64;
                
                // Show progress every 10MB
                if downloaded % (10 * 1024 * 1024) == 0 {
                    if let Some(total) = content_length {
                        let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
                        info!("Download progress: {}% ({:.1}MB / {:.1}MB)", 
                              progress, downloaded as f64 / 1024.0 / 1024.0, 
                              total as f64 / 1024.0 / 1024.0);
                    } else {
                        info!("Downloaded: {:.1}MB", downloaded as f64 / 1024.0 / 1024.0);
                    }
                }
            }
            
            info!("Whisper model downloaded successfully to: {}", self.model_path);
        } else {
            info!("Whisper model found at: {}", self.model_path);
        }
        
        // Now load the model
        info!("Loading Whisper model into memory...");
        self.load_whisper_model().await?;
        self.model_loaded = true;
        info!("Whisper model loaded and ready for transcription");
        
        Ok(())
    }

    pub async fn preload_model(&mut self) -> Result<()> {
        self.load_whisper_model().await?;
        self.model_loaded = true;
        Ok(())
    }

    fn check_model_availability(&self) -> Result<()> {
        // Check if model exists
        if !std::path::Path::new(&self.model_path).exists() {
            warn!("Whisper model not found at: {}", self.model_path);
            warn!("To enable real Whisper STT, download ggml-base.bin from:");
            warn!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin");
            warn!("And place it at: {}", self.model_path);
            return Err(crate::error::VoipGlotError::Configuration(
                "Whisper model not found. Please download manually.".to_string()
            ));
        }
        
        // Check if file has reasonable size
        let metadata = std::fs::metadata(&self.model_path)
            .map_err(|e| crate::error::VoipGlotError::Io(e))?;
        
        if metadata.len() < 1000000 { // Less than 1MB is probably not a real model
            return Err(crate::error::VoipGlotError::Configuration(
                "Whisper model file appears to be invalid or incomplete.".to_string()
            ));
        }
        
        info!("Whisper model file found and appears valid");
        Ok(())
    }

    fn fallback_transcription(&self, audio_data: &[f32]) -> Result<String> {
        // Simple energy-based speech detection as fallback
        let energy: f32 = audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32;
        let rms = energy.sqrt();
        
        debug!("Audio RMS: {}, threshold: 0.01", rms);
        
        if rms > 0.01 {
            // This indicates speech was detected, but we don't have real transcription yet
            debug!("Speech detected (RMS: {}), but Whisper not yet integrated", rms);
            // Return empty string to indicate no transcription available
            Ok(String::new())
        } else {
            Ok(String::new())
        }
    }

    fn preprocess_audio(&self, audio_data: Vec<f32>) -> Vec<f32> {
        // Convert stereo to mono if needed
        let mut mono_audio = if audio_data.len() % 2 == 0 {
            // Assume stereo input
            audio_data
                .chunks(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect()
        } else {
            audio_data
        };
        
        // Normalize audio to prevent clipping
        if let Some(max_val) = mono_audio.iter().map(|x| x.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()) {
            if max_val > 0.0 {
                let scale = (1.0 / max_val).min(1.0);
                for sample in &mut mono_audio {
                    *sample *= scale;
                }
            }
        }
        
        mono_audio
    }

    fn get_whisper_language_code(&self) -> &str {
        // Map language codes to Whisper language codes
        match self.language.as_str() {
            "en" => "en",
            "es" => "es",
            "fr" => "fr",
            "de" => "de",
            "it" => "it",
            "pt" => "pt",
            "ru" => "ru",
            "ja" => "ja",
            "ko" => "ko",
            "zh" => "zh",
            _ => "auto", // Auto-detect for unsupported languages
        }
    }

    pub fn set_language(&mut self, language: String) -> Result<()> {
        self.language = language;
        info!("STT language set to: {} (Offline STT)", self.language);
        Ok(())
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        vec![
            "en".to_string(), // English
            "es".to_string(), // Spanish
            "fr".to_string(), // French
            "de".to_string(), // German
            "it".to_string(), // Italian
            "pt".to_string(), // Portuguese
            "ru".to_string(), // Russian
            "ja".to_string(), // Japanese
            "ko".to_string(), // Korean
            "zh".to_string(), // Chinese
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WhisperResponse {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AzureSttResponse {
    #[serde(rename = "DisplayText")]
    display_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleSttResponse {
    results: Vec<GoogleSttResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleSttResult {
    alternatives: Vec<GoogleSttAlternative>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleSttAlternative {
    transcript: String,
} 