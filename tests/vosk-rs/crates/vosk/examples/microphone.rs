//! Microphone example - clean speech recognition with user-friendly output
//! Run with:
//! cargo run --example microphone <model path> <duration>
//! e.g. "cargo run --example microphone /home/user/stt/model 10"

use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    ChannelCount, SampleFormat,
};
use dasp::{sample::ToSample, Sample};
use vosk::{DecodingState, Model, Recognizer};

fn main() {
    let mut args = env::args();
    args.next();

    let model_path = args.next().expect("A model path was not provided");
    let record_duration = Duration::from_secs(
        args.next()
            .expect("A recording duration was not provided")
            .parse()
            .expect("Invalid recording duration"),
    );

    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, config.sample_rate().0 as f32)
        .expect("Could not create the Recognizer");

    recognizer.set_max_alternatives(10); // Keep multiple alternatives like original
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let recognizer = Arc::new(Mutex::new(recognizer));

    let err_fn = move |err| {
        eprintln!("Audio error: {}", err);
    };

    let recognizer_clone = recognizer.clone();
    let stream = match config.sample_format() {
        SampleFormat::I8 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i8], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
            None,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
            None,
        ),
        SampleFormat::I32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i32], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
            None,
        ),
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| recognize(&mut recognizer_clone.lock().unwrap(), data, channels),
            err_fn,
            None,
        ),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
    .expect("Could not build stream");

    stream.play().expect("Could not play stream");
    println!("🎤 Recording for {} seconds... (speak now)", record_duration.as_secs());
    println!("📝 Partial results will show as you speak:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    std::thread::sleep(record_duration);
    drop(stream);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 Final result:");
    
    let mut recognizer_guard = recognizer.lock().unwrap();
    let final_result = recognizer_guard.final_result();
    
    // Result will always be multiple because we called set_max_alternatives
    if let Some(results) = final_result.multiple() {
        if let Some(best) = results.alternatives.first() {
            if !best.text.is_empty() {
                println!("✅ \"{}\" (confidence: {:.2})", best.text, best.confidence);
            } else {
                println!("❌ No speech detected");
            }
        }
    }
}

fn recognize<T: Sample + ToSample<i16>>(
    recognizer: &mut Recognizer,
    data: &[T],
    channels: ChannelCount,
) {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    let data = if channels != 1 {
        stereo_to_mono(&data)
    } else {
        data
    };

    let state = recognizer.accept_waveform(&data).unwrap();
    match state {
        DecodingState::Running => {
            let partial = recognizer.partial_result();
            // Only show partial results if there's actual text
            if !partial.partial.trim().is_empty() {
                println!("🔄 \"{}\"", partial.partial);
            }
        }
        DecodingState::Finalized => {
            // Result will always be multiple because we called set_max_alternatives
            if let Some(results) = recognizer.result().multiple() {
                if let Some(best) = results.alternatives.first() {
                    if !best.text.is_empty() {
                        println!("✅ \"{}\" (confidence: {:.2})", best.text, best.confidence);
                    }
                }
            }
        }
        DecodingState::Failed => eprintln!("❌ Recognition error"),
    }
}

pub fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );

    result
} 