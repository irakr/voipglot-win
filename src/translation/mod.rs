pub mod stt;
pub mod translator_api;
pub mod tts;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::AppConfig;

use self::stt::STTProcessor;
use self::translator_api::TranslatorProcessor;
use self::tts::TTSProcessor;

pub struct TranslationPipeline {
    stt: STTProcessor,
    translator: TranslatorProcessor,
    tts: TTSProcessor,
    running: bool,
}

impl TranslationPipeline {
    pub fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing translation pipeline");
        
        // Create channels for inter-module communication
        let (stt_text_tx, stt_text_rx) = mpsc::unbounded_channel::<String>();
        let (translator_text_tx, translator_text_rx) = mpsc::unbounded_channel::<String>();
        let (tts_audio_tx, tts_audio_rx) = mpsc::unbounded_channel::<Vec<f32>>();
        
        // Initialize STT processor
        let stt = STTProcessor::new(config.clone(), stt_text_tx)?;
        
        // Initialize translator processor
        let translator = TranslatorProcessor::new(config.clone(), translator_text_tx)?;
        
        // Initialize TTS processor
        let tts = TTSProcessor::new(config.clone(), tts_audio_tx)?;
        
        // Start pipeline processing tasks
        Self::start_pipeline_tasks(stt_text_rx, translator_text_rx, tts_audio_rx, translator.clone());
        
        Ok(Self {
            stt,
            translator,
            tts,
            running: false,
        })
    }
    
    fn start_pipeline_tasks(
        mut stt_text_rx: mpsc::UnboundedReceiver<String>,
        mut translator_text_rx: mpsc::UnboundedReceiver<String>,
        mut tts_audio_rx: mpsc::UnboundedReceiver<Vec<f32>>,
        mut translator: TranslatorProcessor,
    ) {
        // STT -> Translator task
        tokio::spawn(async move {
            while let Some(transcribed_text) = stt_text_rx.recv().await {
                info!("STT -> Translator: \"{}\"", transcribed_text);
                
                // Send to translator
                if let Err(e) = translator.process_translation_pipeline(transcribed_text) {
                    error!("Failed to process translation: {}", e);
                }
            }
        });
        
        // Translator -> TTS task
        tokio::spawn(async move {
            while let Some(translated_text) = translator_text_rx.recv().await {
                info!("Translator -> TTS (bypassed text): \"{}\"", translated_text);
                // TODO: Send to TTS when ready
                // tts.process_tts_pipeline(translated_text).await;
            }
        });
        
        // TTS -> Audio Output task (placeholder for now)
        tokio::spawn(async move {
            while let Some(audio_data) = tts_audio_rx.recv().await {
                info!("TTS -> Audio Output: {} samples", audio_data.len());
                // TODO: Send to audio playback when ready
            }
        });
    }
    
    pub fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        info!("Starting translation pipeline");
        
        // Start STT audio capture
        self.stt.start_audio_capture()?;
        
        self.running = true;
        info!("Translation pipeline started successfully");
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        if !self.running {
            return;
        }
        
        info!("Stopping translation pipeline");
        
        self.stt.stop();
        self.running = false;
        
        info!("Translation pipeline stopped");
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
}
