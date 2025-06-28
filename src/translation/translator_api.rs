use crate::error::{Result, VoipGlotError};
use tracing::{info, error, debug};
use serde::{Deserialize, Serialize};

pub struct TranslationApi {
    provider: TranslationProvider,
    client: reqwest::Client,
    api_keys: TranslationApiKeys,
}

#[derive(Debug, Clone)]
pub enum TranslationProvider {
    DeepL,
    Google,
    Azure,
}

#[derive(Debug, Clone)]
pub struct TranslationApiKeys {
    pub deepl: Option<String>,
    pub google: Option<String>,
    pub azure: Option<String>,
}

impl TranslationApi {
    pub fn new() -> Result<Self> {
        info!("Initializing Translation API");
        
        let client = reqwest::Client::new();
        
        // Load API keys from environment variables
        let api_keys = TranslationApiKeys {
            deepl: std::env::var("DEEPL_API_KEY").ok(),
            google: std::env::var("GOOGLE_API_KEY").ok(),
            azure: std::env::var("AZURE_TRANSLATOR_KEY").ok(),
        };
        
        Ok(Self {
            provider: TranslationProvider::DeepL, // Default to DeepL
            client,
            api_keys,
        })
    }

    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }
        
        debug!("Translating text: '{}' from {} to {}", text, source_lang, target_lang);
        
        match self.provider {
            TranslationProvider::DeepL => {
                self.translate_with_deepl(text, source_lang, target_lang).await
            }
            TranslationProvider::Google => {
                self.translate_with_google(text, source_lang, target_lang).await
            }
            TranslationProvider::Azure => {
                self.translate_with_azure(text, source_lang, target_lang).await
            }
        }
    }

    async fn translate_with_deepl(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        debug!("Using DeepL for translation");
        
        let api_key = self.api_keys.deepl.as_ref()
            .ok_or_else(|| VoipGlotError::Api("DeepL API key not found".to_string()))?;
        
        let url = "https://api-free.deepl.com/v2/translate";
        
        let params = [
            ("auth_key", api_key),
            ("text", text),
            ("source_lang", source_lang),
            ("target_lang", target_lang),
        ];
        
        let response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoipGlotError::Api(format!("DeepL API error: {}", error_text)));
        }
        
        let translation_response: DeepLResponse = response.json().await
            .map_err(|e| VoipGlotError::Serialization(e))?;
        
        Ok(translation_response.translations[0].text.clone())
    }

    async fn translate_with_google(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        debug!("Using Google Translate for translation");
        
        let api_key = self.api_keys.google.as_ref()
            .ok_or_else(|| VoipGlotError::Api("Google API key not found".to_string()))?;
        
        let url = "https://translation.googleapis.com/language/translate/v2";
        
        let params = [
            ("key", api_key),
            ("q", text),
            ("source", source_lang),
            ("target", target_lang),
        ];
        
        let response = self.client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoipGlotError::Api(format!("Google Translate API error: {}", error_text)));
        }
        
        let translation_response: GoogleTranslateResponse = response.json().await
            .map_err(|e| VoipGlotError::Serialization(e))?;
        
        Ok(translation_response.data.translations[0].translated_text.clone())
    }

    async fn translate_with_azure(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        debug!("Using Azure Translator for translation");
        
        let api_key = self.api_keys.azure.as_ref()
            .ok_or_else(|| VoipGlotError::Api("Azure Translator API key not found".to_string()))?;
        
        let url = "https://api.cognitive.microsofttranslator.com/translate";
        
        let params = [
            ("api-version", "3.0"),
            ("from", source_lang),
            ("to", target_lang),
        ];
        
        let request_body = vec![AzureTranslateRequest {
            text: text.to_string(),
        }];
        
        let response = self.client
            .post(url)
            .query(&params)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| VoipGlotError::Network(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoipGlotError::Api(format!("Azure Translator API error: {}", error_text)));
        }
        
        let translation_response: Vec<AzureTranslateResponse> = response.json().await
            .map_err(|e| VoipGlotError::Serialization(e))?;
        
        Ok(translation_response[0].translations[0].text.clone())
    }

    pub fn set_provider(&mut self, provider: TranslationProvider) {
        self.provider = provider;
        info!("Translation provider set to: {:?}", self.provider);
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
            "ar".to_string(), // Arabic
            "hi".to_string(), // Hindi
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeepLTranslation {
    text: String,
    detected_source_language: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTranslateResponse {
    data: GoogleTranslateData,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTranslateData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTranslation {
    translated_text: String,
    detected_source_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AzureTranslateRequest {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AzureTranslateResponse {
    translations: Vec<AzureTranslation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AzureTranslation {
    text: String,
    to: String,
} 