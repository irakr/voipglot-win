use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use std::sync::{Arc, atomic::AtomicBool};

use crate::config::AppConfig;

mod stt;
mod translator_api;
mod tts;

pub use stt::STTProcessor;
pub use translator_api::TranslatorProcessor;
pub use tts::TTSProcessor;

pub struct TranslationPipeline {
    stt: STTProcessor,
    translator: TranslatorProcessor,
    running: bool,
    tts_playing: Arc<AtomicBool>,
}

impl TranslationPipeline {
    pub fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing translation pipeline");
        
        // Create shared state to coordinate STT and TTS to prevent audio feedback
        let tts_playing = Arc::new(AtomicBool::new(false));
        
        info!(
            "Audio feedback prevention: {}, TTS queue size: {}, TTS timeout: {}s",
            config.processing.enable_feedback_prevention,
            config.processing.tts_queue_size,
            config.tts.synthesis_timeout_secs
        );
        
        // Create channels for inter-module communication
        let (stt_text_tx, stt_text_rx) = mpsc::unbounded_channel::<String>();
        let (translator_text_tx, translator_text_rx) = mpsc::unbounded_channel::<String>();
        // Use bounded channel for TTS to prevent queue backlog during slow synthesis
        let tts_queue_size = config.processing.tts_queue_size;
        let (tts_text_tx, tts_text_rx) = mpsc::channel::<String>(tts_queue_size);
        
        // Initialize STT processor with feedback prevention state
        let stt = STTProcessor::new(config.clone(), stt_text_tx, tts_playing.clone())?;
        
        // Initialize translator processor
        let translator = TranslatorProcessor::new(config.clone(), translator_text_tx)?;
        
        // Initialize TTS processor with feedback prevention state
        let mut tts = TTSProcessor::new(config.clone(), tts_text_rx, tts_playing.clone())?;
        
        // Start TTS processing in background task
        tokio::spawn(async move {
            if let Err(e) = tts.start_processing().await {
                error!("TTS processing error: {}", e);
            }
        });
        
        // Start pipeline processing tasks  
        Self::start_pipeline_tasks(stt_text_rx, translator_text_rx, tts_text_tx, translator.clone());
        
        Ok(Self {
            stt,
            translator,
            running: false,
            tts_playing,
        })
    }
    
    fn start_pipeline_tasks(
        mut stt_text_rx: mpsc::UnboundedReceiver<String>,
        mut translator_text_rx: mpsc::UnboundedReceiver<String>,
        tts_text_tx: mpsc::Sender<String>,
        mut translator: TranslatorProcessor,
    ) {
        // Task 1: STT text → Translator
        tokio::spawn(async move {
            info!("Starting STT → Translator pipeline task");
            while let Some(text) = stt_text_rx.recv().await {
                debug!("Received text from STT: \"{}\"", text);
                if let Err(e) = translator.process_translation_pipeline(text) {
                    error!("Translation pipeline error: {}", e);
                }
                // Add small yield to ensure responsiveness
                tokio::task::yield_now().await;
            }
            info!("STT → Translator pipeline task ended");
        });
        
        // Task 2: Translated text → TTS
        tokio::spawn(async move {
            info!("Starting Translator → TTS pipeline task");
            while let Some(translated_text) = translator_text_rx.recv().await {
                debug!("Received translated text: \"{}\"", translated_text);
                
                // Use try_send to handle bounded channel - drop request if TTS queue is full
                match tts_text_tx.try_send(translated_text.clone()) {
                    Ok(_) => {
                        debug!("Text sent to TTS queue successfully");
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                        warn!("TTS queue is full, dropping request: \"{}\" (preventing backlog during slow synthesis)", translated_text);
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                        error!("TTS channel is closed, stopping translator pipeline");
                        break;
                    }
                }
                
                // Add small yield to ensure responsiveness
                tokio::task::yield_now().await;
            }
            info!("Translator → TTS pipeline task ended");
        });
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        self.running = true;
        info!("Starting translation pipeline");
        
        // Start STT processing (starts audio capture and runs indefinitely)  
        self.stt.start_processing().await?;
        
        self.running = false;
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        
        info!("Stopping translation pipeline");
        self.running = false;
        
        // Individual processors will be stopped when their channels are closed
        // This happens automatically when the TranslationPipeline is dropped
        
        Ok(())
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Drop for TranslationPipeline {
    fn drop(&mut self) {
        if self.running {
            info!("Translation pipeline dropped while running, stopping...");
            // Channels will be automatically closed, stopping the processors
        }
    }
}
