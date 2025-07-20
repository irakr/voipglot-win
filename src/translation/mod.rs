use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

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
}

impl TranslationPipeline {
    pub fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing translation pipeline");
        
        // Create channels for inter-module communication
        let (stt_text_tx, stt_text_rx) = mpsc::unbounded_channel::<String>();
        let (translator_text_tx, translator_text_rx) = mpsc::unbounded_channel::<String>();
        let (tts_text_tx, tts_text_rx) = mpsc::unbounded_channel::<String>();
        
        // Initialize STT processor
        let stt = STTProcessor::new(config.clone(), stt_text_tx)?;
        
        // Initialize translator processor
        let translator = TranslatorProcessor::new(config.clone(), translator_text_tx)?;
        
        // Initialize TTS processor (now takes text input channel)
        let mut tts = TTSProcessor::new(config.clone(), tts_text_rx)?;
        
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
        })
    }
    
    fn start_pipeline_tasks(
        mut stt_text_rx: mpsc::UnboundedReceiver<String>,
        mut translator_text_rx: mpsc::UnboundedReceiver<String>,
        tts_text_tx: mpsc::UnboundedSender<String>,
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
            }
            info!("STT → Translator pipeline task ended");
        });
        
        // Task 2: Translated text → TTS
        tokio::spawn(async move {
            info!("Starting Translator → TTS pipeline task");
            while let Some(translated_text) = translator_text_rx.recv().await {
                debug!("Received translated text: \"{}\"", translated_text);
                if let Err(e) = tts_text_tx.send(translated_text) {
                    error!("Failed to send text to TTS: {}", e);
                }
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
