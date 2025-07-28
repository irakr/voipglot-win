//! # VoipGlot Windows Tauri Library
//! 
//! This library provides the Tauri backend for the VoipGlot Windows application.
//! It integrates the voipglot-core library with Tauri for GUI functionality.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{App, Manager};
use voipglot_core::{VoipGlotPipeline, PipelineConfig};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

// State management
pub struct AppState {
    pipeline: Mutex<Option<VoipGlotPipeline>>,
    is_running: AtomicBool,
    config: Mutex<PipelineConfig>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pipeline: Mutex::new(None),
            is_running: AtomicBool::new(false),
            config: Mutex::new(PipelineConfig::default()),
        }
    }
}

// Data structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevices {
    input: Vec<String>,
    output: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Language {
    code: String,
    name: String,
}

// Command handlers module
pub mod commands {
    use super::*;
    
    #[tauri::command]
    pub async fn start_audio_processing(
        state: tauri::State<'_, AppState>,
        input_device: String,
        output_device: String,
        source_language: String,
        target_language: String,
    ) -> Result<(), String> {
        // Update configuration
        {
            let mut config = state.config.lock().map_err(|e| e.to_string())?;
            config.audio.input.input_device = Some(input_device);
            config.audio.output.output_device = Some(output_device);
            config.translation.source_language = source_language;
            config.translation.target_language = target_language;
        } // config guard is dropped here
        
        // Create pipeline
        let config = state.config.lock().map_err(|e| e.to_string())?.clone();
        let pipeline = VoipGlotPipeline::new(config)
            .map_err(|e| format!("Failed to create pipeline: {}", e))?;
        
        // Store pipeline in state
        {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| e.to_string())?;
            *pipeline_guard = Some(pipeline);
        } // pipeline_guard is dropped here
        
        // Start the pipeline - extract pipeline and drop guard before async
        let mut pipeline = {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| e.to_string())?;
            pipeline_guard.take().ok_or("No pipeline found")?
        }; // pipeline_guard is dropped here, we own the pipeline
        
        // Call the async method on the owned pipeline
        pipeline.start().await.map_err(|e| format!("Failed to start pipeline: {}", e))?;
        state.is_running.store(true, Ordering::SeqCst);
        
        // Put the pipeline back
        {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| e.to_string())?;
            *pipeline_guard = Some(pipeline);
        }
        
        Ok(())
    }

    #[tauri::command]
    pub async fn stop_audio_processing(state: tauri::State<'_, AppState>) -> Result<(), String> {
        // Stop the pipeline - extract pipeline and drop guard before async
        let mut pipeline = {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| e.to_string())?;
            pipeline_guard.take().ok_or("No pipeline found")?
        }; // pipeline_guard is dropped here, we own the pipeline
        
        // Call the async method on the owned pipeline
        pipeline.stop().await.map_err(|e| format!("Failed to stop pipeline: {}", e))?;
        state.is_running.store(false, Ordering::SeqCst);
        
        // Put the pipeline back
        {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| e.to_string())?;
            *pipeline_guard = Some(pipeline);
        }
        
        Ok(())
    }

    #[tauri::command]
    pub async fn get_audio_devices() -> Result<AudioDevices, String> {
        let input_devices = voipglot_core::audio::list_input_devices()
            .map_err(|e| format!("Failed to list input devices: {}", e))?;
        
        let output_devices = voipglot_core::audio::list_output_devices()
            .map_err(|e| format!("Failed to list output devices: {}", e))?;
        
        Ok(AudioDevices {
            input: input_devices.into_iter().map(|d| d.name).collect(),
            output: output_devices.into_iter().map(|d| d.name).collect(),
        })
    }

    #[tauri::command]
    pub fn get_supported_languages() -> Vec<Language> {
        vec![
            Language { code: "en".to_string(), name: "English".to_string() },
            Language { code: "es".to_string(), name: "Spanish".to_string() },
            Language { code: "fr".to_string(), name: "French".to_string() },
            Language { code: "de".to_string(), name: "German".to_string() },
            Language { code: "it".to_string(), name: "Italian".to_string() },
            Language { code: "pt".to_string(), name: "Portuguese".to_string() },
            Language { code: "ru".to_string(), name: "Russian".to_string() },
            Language { code: "ja".to_string(), name: "Japanese".to_string() },
            Language { code: "ko".to_string(), name: "Korean".to_string() },
            Language { code: "zh".to_string(), name: "Chinese".to_string() },
        ]
    }

    #[tauri::command]
    pub fn is_processing_active(state: tauri::State<'_, AppState>) -> bool {
        state.is_running.load(Ordering::SeqCst)
    }
}

// Tauri app setup
pub fn create_app() -> App {
    use commands::*;
    
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            start_audio_processing,
            stop_audio_processing,
            get_audio_devices,
            get_supported_languages,
            is_processing_active,
        ])
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                let window = _app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
} 