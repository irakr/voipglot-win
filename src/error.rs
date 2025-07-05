use thiserror::Error;

#[derive(Error, Debug)]
pub enum VoipGlotError {
    #[error("Audio error: {0}")]
    Audio(String),
    
    #[error("Translation error: {0}")]
    Translation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Invalid audio format: {0}")]
    InvalidAudioFormat(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<cpal::BuildStreamError> for VoipGlotError {
    fn from(err: cpal::BuildStreamError) -> Self {
        VoipGlotError::Audio(format!("Failed to build audio stream: {}", err))
    }
}

impl From<cpal::PlayStreamError> for VoipGlotError {
    fn from(err: cpal::PlayStreamError) -> Self {
        VoipGlotError::Audio(format!("Failed to play audio stream: {}", err))
    }
}

impl From<cpal::DefaultStreamConfigError> for VoipGlotError {
    fn from(err: cpal::DefaultStreamConfigError) -> Self {
        VoipGlotError::Audio(format!("Failed to get default stream config: {}", err))
    }
}

impl From<cpal::PauseStreamError> for VoipGlotError {
    fn from(err: cpal::PauseStreamError) -> Self {
        VoipGlotError::Audio(format!("Failed to pause audio stream: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, VoipGlotError>; 