//! # VoipGlot Windows Tauri Library
//! 
//! This library provides the Tauri backend for the VoipGlot Windows application.
//! It integrates the voipglot-core library with Tauri for GUI functionality.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{App, Manager};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::env;
use std::path::Path;
use tracing::{info, warn, error, debug};

// Import voipglot_core
use voipglot_core::{VoipGlotPipeline, PipelineConfig};

// State management
pub struct AppState {
    pipeline: Mutex<Option<VoipGlotPipeline>>,
    is_running: AtomicBool,
    config: Mutex<PipelineConfig>,
}

impl AppState {
    pub fn new() -> Self {
        // Try to load configuration from file, fall back to default if not found
        // Check multiple possible locations for config.toml
        let config_paths = [
            "config.toml",                    // Current directory
            "../config.toml",                 // Parent directory (when running from src-tauri)
            "../../config.toml",              // Two levels up (fallback)
        ];
        
        let mut config = None;
        let mut loaded_path = None;
        
        for path in &config_paths {
            match PipelineConfig::load(path) {
                Ok(cfg) => {
                    info!("Configuration loaded successfully from {}", path);
                    info!("Initial STT model path: {}", cfg.stt.model_path);
                    info!("Initial translation model path: {}", cfg.translation.model_path);
                    info!("Initial TTS model path: {}", cfg.tts.model_path);
                    config = Some(cfg);
                    loaded_path = Some(path.to_string());
                    break;
                }
                Err(e) => {
                    debug!("Failed to load config from {}: {}", path, e);
                }
            }
        }
        
        let mut config = config.unwrap_or_else(|| {
            warn!("Failed to load config.toml from any location. Using default configuration.");
            let default_config = PipelineConfig::default();
            info!("Default STT model path: {}", default_config.stt.model_path);
            info!("Default translation model path: {}", default_config.translation.model_path);
            info!("Default TTS model path: {}", default_config.tts.model_path);
            default_config
        });
        
        if let Some(path) = loaded_path {
            info!("Using configuration loaded from: {}", path);
        }
        
        // Resolve model paths relative to the executable location
        Self::resolve_model_paths(&mut config);
        
        Self {
            pipeline: Mutex::new(None),
            is_running: AtomicBool::new(false),
            config: Mutex::new(config),
        }
    }
    
    fn resolve_model_paths(config: &mut PipelineConfig) {
        // Get the executable directory
        let exe_path = std::env::current_exe().unwrap_or_else(|_| {
            warn!("Failed to get executable path, using current directory");
            std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
        });
        let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
        
        info!("Executable directory: {:?}", exe_dir);
        
        // Resolve STT model path (resources/models/...)
        let stt_path = exe_dir.join(&config.stt.model_path);
        if stt_path.exists() {
            config.stt.model_path = stt_path.to_string_lossy().to_string();
            info!("STT model path resolved to: {}", config.stt.model_path);
        } else {
            warn!("STT model not found at: {:?}", stt_path);
        }
        
        // Resolve translation model path (resources/models/...)
        let translation_path = exe_dir.join(&config.translation.model_path);
        if translation_path.exists() {
            config.translation.model_path = translation_path.to_string_lossy().to_string();
            info!("Translation model path resolved to: {}", config.translation.model_path);
        } else {
            warn!("Translation model not found at: {:?}", translation_path);
        }
        
        // Resolve TTS model path (resources/models/...)
        let tts_path = exe_dir.join(&config.tts.model_path);
        if tts_path.exists() {
            config.tts.model_path = tts_path.to_string_lossy().to_string();
            info!("TTS model path resolved to: {}", config.tts.model_path);
        } else {
            warn!("TTS model not found at: {:?}", tts_path);
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
    use tracing::{info, warn, error, debug};
    
    #[tauri::command]
    pub async fn start_audio_processing(
        state: tauri::State<'_, AppState>,
        input_device: String,
        output_device: String,
        source_language: String,
        target_language: String,
    ) -> Result<(), String> {
        info!("Starting audio processing with config: input={}, output={}, source={}, target={}", 
              input_device, output_device, source_language, target_language);
        
        // Update configuration
        {
            let mut config = state.config.lock().map_err(|e| {
                error!("Failed to lock config: {}", e);
                e.to_string()
            })?;
            config.audio.input.input_device = Some(input_device.clone());
            config.audio.output.output_device = Some(output_device.clone());
            config.translation.source_language = source_language.clone();
            config.translation.target_language = target_language.clone();
            info!("Configuration updated successfully");
        } // config guard is dropped here
        
        // Create pipeline
        let config = state.config.lock().map_err(|e| {
            error!("Failed to lock config for pipeline creation: {}", e);
            e.to_string()
        })?.clone();
        
        info!("Creating VoipGlot pipeline...");
        let pipeline = VoipGlotPipeline::new(config)
            .map_err(|e| {
                error!("Failed to create pipeline: {}", e);
                format!("Failed to create pipeline: {}", e)
            })?;
        info!("Pipeline created successfully");
        
        // Store pipeline in state
        {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                error!("Failed to lock pipeline state: {}", e);
                e.to_string()
            })?;
            *pipeline_guard = Some(pipeline);
            info!("Pipeline stored in state");
        } // pipeline_guard is dropped here
        
        // Start the pipeline - extract pipeline and drop guard before async
        let mut pipeline = {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                error!("Failed to lock pipeline for start: {}", e);
                e.to_string()
            })?;
            pipeline_guard.take().ok_or_else(|| {
                error!("No pipeline found in state");
                "No pipeline found".to_string()
            })?
        }; // pipeline_guard is dropped here, we own the pipeline
        
        // Call the async method on the owned pipeline
        info!("Starting pipeline...");
        pipeline.start().await.map_err(|e| {
            error!("Failed to start pipeline: {}", e);
            format!("Failed to start pipeline: {}", e)
        })?;
        state.is_running.store(true, Ordering::SeqCst);
        info!("Pipeline started successfully, is_running set to true");
        
        // Put the pipeline back
        {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                error!("Failed to lock pipeline for storage: {}", e);
                e.to_string()
            })?;
            *pipeline_guard = Some(pipeline);
            info!("Pipeline stored back in state");
        }
        
        info!("Audio processing started successfully");
        Ok(())
    }

    #[tauri::command]
    pub async fn stop_audio_processing(state: tauri::State<'_, AppState>) -> Result<(), String> {
        info!("Stopping audio processing...");
        
        // Stop the pipeline - extract pipeline and drop guard before async
        let mut pipeline = {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                error!("Failed to lock pipeline for stop: {}", e);
                e.to_string()
            })?;
            pipeline_guard.take().ok_or_else(|| {
                error!("No pipeline found in state for stop");
                "No pipeline found".to_string()
            })?
        }; // pipeline_guard is dropped here, we own the pipeline
        
        // Call the async method on the owned pipeline
        info!("Stopping pipeline...");
        pipeline.stop().await.map_err(|e| {
            error!("Failed to stop pipeline: {}", e);
            format!("Failed to stop pipeline: {}", e)
        })?;
        state.is_running.store(false, Ordering::SeqCst);
        info!("Pipeline stopped successfully, is_running set to false");
        
        // Put the pipeline back
        {
            let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                error!("Failed to lock pipeline for storage after stop: {}", e);
                e.to_string()
            })?;
            *pipeline_guard = Some(pipeline);
            info!("Pipeline stored back in state after stop");
        }
        
        info!("Audio processing stopped successfully");
        Ok(())
    }

    #[tauri::command]
    pub async fn get_audio_devices() -> Result<AudioDevices, String> {
        info!("Getting audio devices...");
        
        let input_devices = voipglot_core::audio::list_input_devices()
            .map_err(|e| {
                error!("Failed to list input devices: {}", e);
                format!("Failed to list input devices: {}", e)
            })?;
        
        let output_devices = voipglot_core::audio::list_output_devices()
            .map_err(|e| {
                error!("Failed to list output devices: {}", e);
                format!("Failed to list output devices: {}", e)
            })?;
        
        let devices = AudioDevices {
            input: input_devices.into_iter().map(|d| d.name).collect(),
            output: output_devices.into_iter().map(|d| d.name).collect(),
        };
        
        info!("Found {} input devices and {} output devices", devices.input.len(), devices.output.len());
        debug!("Input devices: {:?}", devices.input);
        debug!("Output devices: {:?}", devices.output);
        
        Ok(devices)
    }

    #[tauri::command]
    pub fn get_supported_languages() -> Vec<Language> {
        info!("Getting supported languages...");
        
        let languages = vec![
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
        ];
        
        info!("Returning {} supported languages", languages.len());
        debug!("Languages: {:?}", languages);
        
        languages
    }

    #[tauri::command]
    pub fn is_processing_active(state: tauri::State<'_, AppState>) -> bool {
        let is_active = state.is_running.load(Ordering::SeqCst);
        debug!("Checking if processing is active: {}", is_active);
        is_active
    }

    #[tauri::command]
    pub fn get_audio_frequency_data(state: tauri::State<'_, AppState>) -> Vec<f32> {
        let is_active = state.is_running.load(Ordering::SeqCst);
        debug!("Getting audio frequency data, processing active: {}", is_active);
        
        // TODO: Get real frequency data from the pipeline
        // For now, return mock data
        let mut frequencies = Vec::new();
        for _ in 0..15 {
            frequencies.push(rand::random::<f32>() * 0.8 + 0.1); // 0.1 to 0.9
        }
        
        debug!("Generated {} frequency values", frequencies.len());
        frequencies
    }

    #[tauri::command]
    pub async fn update_configuration(
        state: tauri::State<'_, AppState>,
        input_device: Option<String>,
        output_device: Option<String>,
        source_language: Option<String>,
        target_language: Option<String>,
    ) -> Result<(), String> {
        info!("Updating configuration: input={:?}, output={:?}, source={:?}, target={:?}", 
              input_device, output_device, source_language, target_language);
        
        let was_running = state.is_running.load(Ordering::SeqCst);
        info!("Processing was running: {}", was_running);
        
        // Stop if running
        if was_running {
            info!("Stopping pipeline for configuration update...");
            let mut pipeline = {
                let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                    error!("Failed to lock pipeline for update stop: {}", e);
                    e.to_string()
                })?;
                pipeline_guard.take().ok_or_else(|| {
                    error!("No pipeline found for update stop");
                    "No pipeline found".to_string()
                })?
            };
            
            pipeline.stop().await.map_err(|e| {
                error!("Failed to stop pipeline for update: {}", e);
                format!("Failed to stop pipeline for update: {}", e)
            })?;
            state.is_running.store(false, Ordering::SeqCst);
            info!("Pipeline stopped for configuration update");
            
            {
                let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                    error!("Failed to lock pipeline for storage after update stop: {}", e);
                    e.to_string()
                })?;
                *pipeline_guard = Some(pipeline);
            }
        }
        
        // Update configuration
        {
            let mut config = state.config.lock().map_err(|e| {
                error!("Failed to lock config for update: {}", e);
                e.to_string()
            })?;
            if let Some(input) = input_device {
                config.audio.input.input_device = Some(input);
                info!("Updated input device");
            }
            if let Some(output) = output_device {
                config.audio.output.output_device = Some(output);
                info!("Updated output device");
            }
            if let Some(source) = source_language {
                config.translation.source_language = source;
                info!("Updated source language");
            }
            if let Some(target) = target_language {
                config.translation.target_language = target;
                info!("Updated target language");
            }
        }
        
        // Restart if was running
        if was_running {
            info!("Restarting pipeline with new configuration...");
            let config = state.config.lock().map_err(|e| {
                error!("Failed to lock config for restart: {}", e);
                e.to_string()
            })?.clone();
            
            let pipeline = VoipGlotPipeline::new(config)
                .map_err(|e| {
                    error!("Failed to create pipeline for restart: {}", e);
                    format!("Failed to create pipeline for restart: {}", e)
                })?;
            
            {
                let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                    error!("Failed to lock pipeline for restart storage: {}", e);
                    e.to_string()
                })?;
                *pipeline_guard = Some(pipeline);
            }
            
            let mut pipeline = {
                let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                    error!("Failed to lock pipeline for restart start: {}", e);
                    e.to_string()
                })?;
                pipeline_guard.take().ok_or_else(|| {
                    error!("No pipeline found for restart start");
                    "No pipeline found".to_string()
                })?
            };
            
            pipeline.start().await.map_err(|e| {
                error!("Failed to start pipeline for restart: {}", e);
                format!("Failed to start pipeline for restart: {}", e)
            })?;
            state.is_running.store(true, Ordering::SeqCst);
            info!("Pipeline restarted successfully");
            
            {
                let mut pipeline_guard = state.pipeline.lock().map_err(|e| {
                    error!("Failed to lock pipeline for restart storage: {}", e);
                    e.to_string()
                })?;
                *pipeline_guard = Some(pipeline);
            }
        }
        
        info!("Configuration updated successfully");
        Ok(())
    }

    #[tauri::command]
    pub fn test_connection() -> String {
        info!("Test connection command called");
        "Tauri backend is working!".to_string()
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
            get_audio_frequency_data,
            update_configuration,
            test_connection,
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