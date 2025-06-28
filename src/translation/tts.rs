use crate::error::{Result, VoipGlotError};
use tracing::{info, error, debug};
use serde::{Deserialize, Serialize};

pub struct TextToSpeech {
    language: String,
    provider: TtsProvider,
    client: reqwest::Client,
    api_keys: TtsApiKeys,
}

#[derive(Debug, Clone)]
pub enum TtsProvider {
    Azure,
    ElevenLabs,
    Google,
}

#[derive(Debug, Clone)]
pub struct TtsApiKeys {
    pub azure: Option<String>,
    pub elevenlabs: Option<String>,
    pub google: Option<String>,
}

impl TextToSpeech {
    pub fn new(language: String) -> Result<Self> {
        info!("Initializing Text-to-Speech with language: {}", language);
        
        let client = reqwest::Client::new();
        
        // Load API keys from environment variables
        let api_keys = TtsApiKeys {
            azure: std::env::var("AZURE_SPEECH_KEY").ok(),
            elevenlabs: std::env::var("ELEVENLABS_API_KEY").ok(),
            google: std::env::var("GOOGLE_API_KEY").ok(),
        };
        
        Ok(Self {
            language,
            provider: TtsProvider::Azure, // Default to Azure
            client,
            api_keys,
        })
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Synthesizing text to speech: '{}'", text);
        
        match self.provider {
            TtsProvider::Azure => self.synthesize_with_azure(text).await,
            TtsProvider::ElevenLabs => self.synthesize_with_elevenlabs(text).await,
            TtsProvider::Google => self.synthesize_with_google(text).await,
        }
    }

    async fn synthesize_with_azure(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Using Azure Speech Services for TTS");
        
        let api_key = self.api_keys.azure.as_ref()
            .ok_or_else(|| VoipGlotError::Api("Azure Speech API key not found".to_string()))?;
        
        let region = std::env::var("AZURE_REGION")
            .unwrap_or_else(|_| "eastus".to_string());
        
        let url = format!("https://{}.tts.speech.microsoft.com/cognitiveservices/v1", region);
        
        let ssml = self.build_azure_ssml(text);
        
        let response = self.client
            .post(&url)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .header("Content-Type", "application/ssml+xml")
            .header("X-Microsoft-OutputFormat", "riff-16khz-16bit-mono-pcm")
            .body(ssml)
            .send()
            .await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoipGlotError::Api(format!("Azure TTS API error: {}", error_text)));
        }
        
        let audio_bytes = response.bytes().await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        // Convert audio bytes to f32 samples
        self.convert_audio_bytes_to_samples(&audio_bytes)
    }

    async fn synthesize_with_elevenlabs(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Using ElevenLabs for TTS");
        
        let api_key = self.api_keys.elevenlabs.as_ref()
            .ok_or_else(|| VoipGlotError::Api("ElevenLabs API key not found".to_string()))?;
        
        let url = "https://api.elevenlabs.io/v1/text-to-speech/21m00Tcm4TlvDq8ikWAM";
        
        let request_body = ElevenLabsRequest {
            text: text.to_string(),
            model_id: "eleven_monolingual_v1".to_string(),
            voice_settings: ElevenLabsVoiceSettings {
                stability: 0.5,
                similarity_boost: 0.5,
            },
        };
        
        let response = self.client
            .post(url)
            .header("xi-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoipGlotError::Api(format!("ElevenLabs API error: {}", error_text)));
        }
        
        let audio_bytes = response.bytes().await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        // Convert audio bytes to f32 samples
        self.convert_audio_bytes_to_samples(&audio_bytes)
    }

    async fn synthesize_with_google(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Using Google Text-to-Speech for TTS");
        
        let api_key = self.api_keys.google.as_ref()
            .ok_or_else(|| VoipGlotError::Api("Google API key not found".to_string()))?;
        
        let url = "https://texttospeech.googleapis.com/v1/text:synthesize";
        
        let request_body = GoogleTtsRequest {
            input: GoogleTtsInput {
                text: text.to_string(),
            },
            voice: GoogleTtsVoice {
                language_code: self.language.clone(),
                name: format!("{}-Standard-A", self.language),
            },
            audio_config: GoogleTtsAudioConfig {
                audio_encoding: "LINEAR16".to_string(),
                sample_rate_hertz: 16000,
            },
        };
        
        let response = self.client
            .post(url)
            .query(&[("key", api_key)])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoipGlotError::Api(format!("Google TTS API error: {}", error_text)));
        }
        
        let tts_response: GoogleTtsResponse = response.json().await
            .map_err(|e| VoipGlotError::Serialization(e))?;
        
        // Decode base64 audio content
        let audio_bytes = base64::decode(&tts_response.audio_content)
            .map_err(|e| VoipGlotError::Audio(format!("Failed to decode base64 audio: {}", e)))?;
        
        // Convert audio bytes to f32 samples
        self.convert_audio_bytes_to_samples(&audio_bytes)
    }

    fn build_azure_ssml(&self, text: &str) -> String {
        // Build SSML (Speech Synthesis Markup Language) for Azure
        format!(
            r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="{}">
                <voice name="{}">
                    {}
                </voice>
            </speak>"#,
            self.language,
            self.get_azure_voice_name(),
            text
        )
    }

    fn get_azure_voice_name(&self) -> &str {
        match self.language.as_str() {
            "en" => "en-US-JennyNeural",
            "es" => "es-ES-ElviraNeural",
            "fr" => "fr-FR-DeniseNeural",
            "de" => "de-DE-KatjaNeural",
            "it" => "it-IT-ElsaNeural",
            "pt" => "pt-BR-FranciscaNeural",
            "ru" => "ru-RU-SvetlanaNeural",
            "ja" => "ja-JP-NanamiNeural",
            "ko" => "ko-KR-SunHiNeural",
            "zh" => "zh-CN-XiaoxiaoNeural",
            _ => "en-US-JennyNeural", // Default fallback
        }
    }

    fn convert_audio_bytes_to_samples(&self, audio_bytes: &[u8]) -> Result<Vec<f32>> {
        // Convert 16-bit PCM audio bytes to f32 samples
        let mut samples = Vec::new();
        
        // Skip WAV header if present (first 44 bytes)
        let audio_data = if audio_bytes.len() > 44 && &audio_bytes[0..4] == b"RIFF" {
            &audio_bytes[44..]
        } else {
            audio_bytes
        };
        
        // Convert 16-bit little-endian samples to f32
        for chunk in audio_data.chunks(2) {
            if chunk.len() == 2 {
                let sample_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
                let sample_f32 = sample_i16 as f32 / 32768.0; // Normalize to [-1.0, 1.0]
                samples.push(sample_f32);
            }
        }
        
        Ok(samples)
    }

    pub fn set_language(&mut self, language: String) -> Result<()> {
        self.language = language;
        info!("TTS language set to: {}", self.language);
        Ok(())
    }

    pub fn set_provider(&mut self, provider: TtsProvider) {
        self.provider = provider;
        info!("TTS provider set to: {:?}", self.provider);
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

#[derive(Debug, Serialize)]
struct ElevenLabsRequest {
    text: String,
    model_id: String,
    voice_settings: ElevenLabsVoiceSettings,
}

#[derive(Debug, Serialize)]
struct ElevenLabsVoiceSettings {
    stability: f32,
    similarity_boost: f32,
}

#[derive(Debug, Serialize)]
struct GoogleTtsRequest {
    input: GoogleTtsInput,
    voice: GoogleTtsVoice,
    audio_config: GoogleTtsAudioConfig,
}

#[derive(Debug, Serialize)]
struct GoogleTtsInput {
    text: String,
}

#[derive(Debug, Serialize)]
struct GoogleTtsVoice {
    language_code: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct GoogleTtsAudioConfig {
    audio_encoding: String,
    sample_rate_hertz: i32,
}

#[derive(Debug, Deserialize)]
struct GoogleTtsResponse {
    audio_content: String,
} 