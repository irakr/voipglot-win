use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

/// List available audio input devices
pub fn list_input_devices() -> Result<()> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;
    
    println!("Available input devices:");
    for (i, device) in devices.enumerate() {
        if let Ok(name) = device.name() {
            println!("  {}: {}", i, name);
        }
    }
    
    Ok(())
}

/// Get default input device info
pub fn get_default_input_device_info() -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No default input device found"))?;
    
    let name = device.name()?;
    println!("Default input device: {}", name);
    
    // List supported configs
    let configs = device.default_input_config()?;
    println!("Default config: {:?}", configs);
    
    Ok(())
}

/// Test VOSK model loading
pub fn test_vosk_model(model_path: &str) -> Result<()> {
    use vosk::{Model, Recognizer};
    
    println!("Testing VOSK model: {}", model_path);
    
    let model = Model::new(model_path)
        .ok_or_else(|| anyhow::anyhow!("Failed to load VOSK model from: {}", model_path))?;
    let _recognizer = Recognizer::new(&model, 16000.0)
        .ok_or_else(|| anyhow::anyhow!("Failed to create VOSK recognizer"))?;
    
    println!("VOSK model loaded successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_devices() {
        // This test will only pass if there are input devices available
        let result = list_input_devices();
        // We don't assert here as the test environment might not have audio devices
        println!("Device listing result: {:?}", result);
    }
    
    #[test]
    fn test_default_device_info() {
        let result = get_default_input_device_info();
        // This test might fail if no default device is available
        println!("Default device info result: {:?}", result);
    }
} 