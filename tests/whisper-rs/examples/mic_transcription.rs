use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting microphone transcription...");
    println!("Press Ctrl+C to stop");

    // Load a context and model.
    let mut context_param = WhisperContextParameters::default();

    // Enable DTW token level timestamp for tiny model
    context_param.dtw_parameters.mode = whisper_rs::DtwMode::ModelPreset {
        model_preset: whisper_rs::DtwModelPreset::TinyEn,
    };

    let ctx = WhisperContext::new_with_params(
        "ggml-tiny.bin",
        context_param,
    )
    .expect("failed to load model");

    // Create a state
    let mut state = ctx.create_state().expect("failed to create state");

    // Create a params object for running the model.
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });

    // Configure params
    params.set_n_threads(1);
    params.set_translate(true);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_token_timestamps(true);

    // Get the default host and device
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or("No input device found")?;

    println!("Using input device: {}", device.name()?);

    // Get the default input config
    let config = device.default_input_config()?;
    println!("Audio config: {:?}", config);

    // Check if we need to resample to 16kHz
    let target_sample_rate = 16000;
    let needs_resampling = config.sample_rate().0 != target_sample_rate;
    let channels = config.channels();

    // Shared buffer for audio data
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let audio_buffer_clone = audio_buffer.clone();

    // Build the input stream
    let input_stream = device.build_input_stream(
        &config.clone().into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut buffer = audio_buffer_clone.lock().unwrap();
            buffer.extend_from_slice(data);
        },
        |err| eprintln!("Audio input error: {}", err),
        None,
    )?;

    // Start the stream
    input_stream.play()?;

    println!("Listening... Speak now!");

    // Main processing loop
    // We need at least 1 second of audio at 16kHz
    // With 48kHz input, we need 3 seconds to get 1 second after resampling
    let chunk_size = config.sample_rate().0 as usize * 3; // 3 seconds of audio at input rate

    loop {
        std::thread::sleep(Duration::from_millis(100));

        let mut buffer = audio_buffer.lock().unwrap();
        
        // Check if we have enough audio data to process
        if buffer.len() >= chunk_size {
            // Extract the chunk to process
            let audio_chunk: Vec<f32> = buffer.drain(..chunk_size).collect();
            let original_len = audio_chunk.len();
            drop(buffer); // Release lock before processing

            // Convert to mono if stereo first
            let mono_audio = if channels > 1 {
                convert_stereo_to_mono(&audio_chunk)?
            } else {
                audio_chunk.clone()
            };

            // Resample if necessary
            let processed_audio = if needs_resampling {
                resample_audio(&mono_audio, config.sample_rate().0, target_sample_rate)?
            } else {
                mono_audio
            };

            println!("Audio chunk: {} samples, processed: {} samples", original_len, processed_audio.len());

            // Process with Whisper
            match state.full(params.clone(), &processed_audio) {
                Ok(_) => {
                    // Get the transcription
                    let num_segments = state.full_n_segments()?;
                    if num_segments > 0 {
                        for i in 0..num_segments {
                            let segment = state.full_get_segment_text(i)?;
                            let start_timestamp = state.full_get_segment_t0(i)?;
                            let end_timestamp = state.full_get_segment_t1(i)?;
                            
                            println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Whisper processing error: {}", e);
                }
            }
        }
    }
}

fn resample_audio(audio: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    if from_rate == to_rate {
        return Ok(audio.to_vec());
    }

    // Simple linear interpolation resampling
    let ratio = from_rate as f32 / to_rate as f32;
    let new_len = (audio.len() as f32 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_index = i as f32 * ratio;
        let src_index_floor = src_index.floor() as usize;
        let src_index_ceil = (src_index.ceil() as usize).min(audio.len() - 1);
        let fraction = src_index - src_index_floor as f32;

        if src_index_floor < audio.len() {
            let sample = if src_index_floor == src_index_ceil {
                audio[src_index_floor]
            } else {
                audio[src_index_floor] * (1.0 - fraction) + audio[src_index_ceil] * fraction
            };
            resampled.push(sample);
        }
    }

    println!("Resampling: {} samples at {}Hz -> {} samples at {}Hz", 
             audio.len(), from_rate, resampled.len(), to_rate);

    Ok(resampled)
}

fn convert_stereo_to_mono(audio: &[f32]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // Assuming stereo audio (2 channels)
    if audio.len() % 2 != 0 {
        return Err("Audio length must be even for stereo conversion".into());
    }

    let mut mono = Vec::with_capacity(audio.len() / 2);
    for i in (0..audio.len()).step_by(2) {
        let left = audio[i];
        let right = audio[i + 1];
        mono.push((left + right) / 2.0);
    }

    Ok(mono)
} 