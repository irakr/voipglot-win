use crate::error::{Result, VoipGlotError};
use tracing::{info, debug, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use lazy_static::lazy_static;

// Offline translator that doesn't require network access
pub struct LocalTranslator {
    translation_cache: Arc<Mutex<HashMap<String, String>>>,
    cache_dir: String,
}

impl LocalTranslator {
    pub fn new() -> Result<Self> {
        info!("Initializing Offline Local Translator");
        
        // Create cache directory for models
        let cache_dir = std::env::var("VOIPGLOT_MODEL_CACHE")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                format!("{}/.voipglot/models", home)
            });
        
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| VoipGlotError::Configuration(format!("Failed to create model cache directory: {}", e)))?;
        
        info!("Model cache directory: {}", cache_dir);
        
        Ok(Self {
            translation_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_dir,
        })
    }

    /// Pre-load the most common translation model to avoid delays during real-time processing
    pub async fn preload_common_model(&self, source_lang: &str, target_lang: &str) -> Result<()> {
        info!("Pre-loading offline translation for {} -> {}", source_lang, target_lang);
        Ok(()) // No actual loading needed for offline translator
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
        
        debug!("Offline translation: '{}' from {} to {}", text, source_lang, target_lang);
        
        // Check cache first
        let cache_key = format!("{}:{}:{}", source_lang, target_lang, text);
        {
            let cache = self.translation_cache.lock().await;
            if let Some(cached_translation) = cache.get(&cache_key) {
                debug!("Using cached translation: '{}'", cached_translation);
                println!("\n🔄 TRANSLATION (cached): '{}' -> '{}'", text, cached_translation);
                println!("🔄 TRANSLATION (cached): '{}' -> '{}'", text, cached_translation);
                println!("🔄 TRANSLATION (cached): '{}' -> '{}'", text, cached_translation);
                return Ok(cached_translation.clone());
            }
        }
        
        // Perform offline translation
        let translation = self.offline_translate(text, source_lang, target_lang)?;
        
        // Cache the result
        {
            let mut cache = self.translation_cache.lock().await;
            cache.insert(cache_key, translation.clone());
        }
        
        debug!("Offline translation result: '{}'", translation);
        println!("\n🔄 TRANSLATION: '{}' -> '{}'", text, translation);
        println!("🔄 TRANSLATION: '{}' -> '{}'", text, translation);
        println!("🔄 TRANSLATION: '{}' -> '{}'", text, translation);
        Ok(translation)
    }

    fn offline_translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        let text_lowercase = text.to_lowercase();
        let text_lower = text_lowercase.trim();
        
        // Don't translate system messages
        if text_lower.contains("[speech detected]") || text_lower.contains("[translated]") {
            return Ok(text.to_string());
        }
        
        // English to Spanish translations
        if source_lang == "en" && target_lang == "es" {
            let translation = self.translate_en_to_es(text_lower);
            Ok(translation)
        }
        // Spanish to English translations
        else if source_lang == "es" && target_lang == "en" {
            let translation = self.translate_es_to_en(text_lower);
            Ok(translation)
        }
        // Add more language pairs as needed
        else {
            warn!("Unsupported language pair: {} -> {}", source_lang, target_lang);
            Ok(format!("[Translation not available for {} -> {}]", source_lang, target_lang))
        }
    }

    fn translate_en_to_es(&self, text: &str) -> String {
        // Comprehensive English to Spanish translation dictionary
        let translations = ENGLISH_TO_SPANISH.lock().unwrap();
        
        // Try exact match first
        if let Some(translation) = translations.get(text) {
            return translation.clone();
        }
        
        // Try word-by-word translation for simple phrases
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= 3 {
            let mut translated_words = Vec::new();
            for word in words {
                if let Some(translation) = translations.get(word) {
                    translated_words.push(translation.clone());
                } else {
                    translated_words.push(word.to_string());
                }
            }
            return translated_words.join(" ");
        }
        
        // Fallback for longer phrases
        "[Translation not available]".to_string()
    }

    fn translate_es_to_en(&self, text: &str) -> String {
        // Comprehensive Spanish to English translation dictionary
        let translations = SPANISH_TO_ENGLISH.lock().unwrap();
        
        // Try exact match first
        if let Some(translation) = translations.get(text) {
            return translation.clone();
        }
        
        // Try word-by-word translation for simple phrases
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= 3 {
            let mut translated_words = Vec::new();
            for word in words {
                if let Some(translation) = translations.get(word) {
                    translated_words.push(translation.clone());
                } else {
                    translated_words.push(word.to_string());
                }
            }
            return translated_words.join(" ");
        }
        
        // Fallback for longer phrases
        "[Translation not available]".to_string()
    }

    pub fn get_supported_language_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("en".to_string(), "es".to_string()),
            ("es".to_string(), "en".to_string()),
        ]
    }

    pub fn is_supported(&self, source_lang: &str, target_lang: &str) -> bool {
        (source_lang == "en" && target_lang == "es") || 
        (source_lang == "es" && target_lang == "en")
    }
}

// Comprehensive English to Spanish translation dictionary
lazy_static! {
    static ref ENGLISH_TO_SPANISH: std::sync::Mutex<HashMap<String, String>> = {
        let mut m = HashMap::new();
        
        // Basic greetings and common phrases
        m.insert("hello".to_string(), "hola".to_string());
        m.insert("hi".to_string(), "hola".to_string());
        m.insert("goodbye".to_string(), "adiós".to_string());
        m.insert("bye".to_string(), "adiós".to_string());
        m.insert("good morning".to_string(), "buenos días".to_string());
        m.insert("good afternoon".to_string(), "buenas tardes".to_string());
        m.insert("good night".to_string(), "buenas noches".to_string());
        m.insert("good evening".to_string(), "buenas noches".to_string());
        
        // Politeness
        m.insert("please".to_string(), "por favor".to_string());
        m.insert("thank you".to_string(), "gracias".to_string());
        m.insert("thanks".to_string(), "gracias".to_string());
        m.insert("excuse me".to_string(), "disculpe".to_string());
        m.insert("sorry".to_string(), "lo siento".to_string());
        m.insert("pardon".to_string(), "perdón".to_string());
        
        // Basic responses
        m.insert("yes".to_string(), "sí".to_string());
        m.insert("no".to_string(), "no".to_string());
        m.insert("okay".to_string(), "vale".to_string());
        m.insert("ok".to_string(), "vale".to_string());
        m.insert("maybe".to_string(), "tal vez".to_string());
        m.insert("perhaps".to_string(), "quizás".to_string());
        
        // Questions
        m.insert("how are you".to_string(), "¿cómo estás?".to_string());
        m.insert("what".to_string(), "qué".to_string());
        m.insert("where".to_string(), "dónde".to_string());
        m.insert("when".to_string(), "cuándo".to_string());
        m.insert("why".to_string(), "por qué".to_string());
        m.insert("who".to_string(), "quién".to_string());
        m.insert("how".to_string(), "cómo".to_string());
        
        // Common verbs
        m.insert("is".to_string(), "es".to_string());
        m.insert("are".to_string(), "son".to_string());
        m.insert("am".to_string(), "soy".to_string());
        m.insert("have".to_string(), "tengo".to_string());
        m.insert("has".to_string(), "tiene".to_string());
        m.insert("want".to_string(), "quiero".to_string());
        m.insert("need".to_string(), "necesito".to_string());
        m.insert("can".to_string(), "puedo".to_string());
        m.insert("will".to_string(), "voy a".to_string());
        m.insert("go".to_string(), "ir".to_string());
        m.insert("come".to_string(), "venir".to_string());
        m.insert("see".to_string(), "ver".to_string());
        m.insert("hear".to_string(), "oír".to_string());
        m.insert("speak".to_string(), "hablar".to_string());
        m.insert("talk".to_string(), "hablar".to_string());
        m.insert("say".to_string(), "decir".to_string());
        m.insert("tell".to_string(), "decir".to_string());
        m.insert("know".to_string(), "saber".to_string());
        m.insert("think".to_string(), "pensar".to_string());
        m.insert("understand".to_string(), "entender".to_string());
        m.insert("help".to_string(), "ayudar".to_string());
        m.insert("help me".to_string(), "ayúdame".to_string());
        
        // Common nouns
        m.insert("water".to_string(), "agua".to_string());
        m.insert("food".to_string(), "comida".to_string());
        m.insert("house".to_string(), "casa".to_string());
        m.insert("car".to_string(), "coche".to_string());
        m.insert("time".to_string(), "tiempo".to_string());
        m.insert("day".to_string(), "día".to_string());
        m.insert("night".to_string(), "noche".to_string());
        m.insert("morning".to_string(), "mañana".to_string());
        m.insert("afternoon".to_string(), "tarde".to_string());
        m.insert("evening".to_string(), "noche".to_string());
        m.insert("week".to_string(), "semana".to_string());
        m.insert("month".to_string(), "mes".to_string());
        m.insert("year".to_string(), "año".to_string());
        m.insert("friend".to_string(), "amigo".to_string());
        m.insert("family".to_string(), "familia".to_string());
        m.insert("work".to_string(), "trabajo".to_string());
        m.insert("home".to_string(), "casa".to_string());
        m.insert("money".to_string(), "dinero".to_string());
        m.insert("phone".to_string(), "teléfono".to_string());
        m.insert("computer".to_string(), "computadora".to_string());
        m.insert("book".to_string(), "libro".to_string());
        m.insert("language".to_string(), "idioma".to_string());
        m.insert("word".to_string(), "palabra".to_string());
        m.insert("name".to_string(), "nombre".to_string());
        m.insert("person".to_string(), "persona".to_string());
        m.insert("people".to_string(), "gente".to_string());
        m.insert("man".to_string(), "hombre".to_string());
        m.insert("woman".to_string(), "mujer".to_string());
        m.insert("child".to_string(), "niño".to_string());
        m.insert("children".to_string(), "niños".to_string());
        m.insert("boy".to_string(), "niño".to_string());
        m.insert("girl".to_string(), "niña".to_string());
        
        // Communication phrases
        m.insert("i don't understand".to_string(), "no entiendo".to_string());
        m.insert("speak slowly".to_string(), "habla despacio".to_string());
        m.insert("repeat".to_string(), "repite".to_string());
        m.insert("what did you say".to_string(), "¿qué dijiste?".to_string());
        m.insert("can you repeat that".to_string(), "¿puedes repetir eso?".to_string());
        m.insert("i don't speak spanish".to_string(), "no hablo español".to_string());
        m.insert("do you speak english".to_string(), "¿hablas inglés?".to_string());
        m.insert("how do you say".to_string(), "¿cómo se dice?".to_string());
        
        // Gaming/VOIP specific phrases
        m.insert("ready".to_string(), "listo".to_string());
        m.insert("wait".to_string(), "espera".to_string());
        m.insert("stop".to_string(), "para".to_string());
        m.insert("go".to_string(), "vamos".to_string());
        m.insert("good".to_string(), "bueno".to_string());
        m.insert("bad".to_string(), "malo".to_string());
        m.insert("great".to_string(), "excelente".to_string());
        m.insert("terrible".to_string(), "terrible".to_string());
        m.insert("win".to_string(), "ganar".to_string());
        m.insert("lose".to_string(), "perder".to_string());
        m.insert("game".to_string(), "juego".to_string());
        m.insert("play".to_string(), "jugar".to_string());
        m.insert("team".to_string(), "equipo".to_string());
        m.insert("player".to_string(), "jugador".to_string());
        m.insert("enemy".to_string(), "enemigo".to_string());
        m.insert("attack".to_string(), "atacar".to_string());
        m.insert("defend".to_string(), "defender".to_string());
        m.insert("move".to_string(), "mover".to_string());
        m.insert("run".to_string(), "correr".to_string());
        m.insert("jump".to_string(), "saltar".to_string());
        m.insert("shoot".to_string(), "disparar".to_string());
        m.insert("hit".to_string(), "golpear".to_string());
        m.insert("miss".to_string(), "fallar".to_string());
        m.insert("dead".to_string(), "muerto".to_string());
        m.insert("alive".to_string(), "vivo".to_string());
        m.insert("health".to_string(), "salud".to_string());
        m.insert("ammo".to_string(), "munición".to_string());
        m.insert("weapon".to_string(), "arma".to_string());
        m.insert("gun".to_string(), "pistola".to_string());
        m.insert("knife".to_string(), "cuchillo".to_string());
        m.insert("bomb".to_string(), "bomba".to_string());
        m.insert("grenade".to_string(), "granada".to_string());
        m.insert("cover".to_string(), "cubrir".to_string());
        m.insert("hide".to_string(), "esconder".to_string());
        m.insert("search".to_string(), "buscar".to_string());
        m.insert("find".to_string(), "encontrar".to_string());
        m.insert("look".to_string(), "mirar".to_string());
        m.insert("watch".to_string(), "observar".to_string());
        m.insert("listen".to_string(), "escuchar".to_string());
        m.insert("hear".to_string(), "oír".to_string());
        m.insert("sound".to_string(), "sonido".to_string());
        m.insert("noise".to_string(), "ruido".to_string());
        m.insert("quiet".to_string(), "silencio".to_string());
        m.insert("loud".to_string(), "fuerte".to_string());
        
        std::sync::Mutex::new(m)
    };
}

// Comprehensive Spanish to English translation dictionary
lazy_static! {
    static ref SPANISH_TO_ENGLISH: std::sync::Mutex<HashMap<String, String>> = {
        let mut m = HashMap::new();
        
        // Basic greetings and common phrases
        m.insert("hola".to_string(), "hello".to_string());
        m.insert("adiós".to_string(), "goodbye".to_string());
        m.insert("buenos días".to_string(), "good morning".to_string());
        m.insert("buenas tardes".to_string(), "good afternoon".to_string());
        m.insert("buenas noches".to_string(), "good night".to_string());
        
        // Politeness
        m.insert("por favor".to_string(), "please".to_string());
        m.insert("gracias".to_string(), "thank you".to_string());
        m.insert("disculpe".to_string(), "excuse me".to_string());
        m.insert("lo siento".to_string(), "sorry".to_string());
        m.insert("perdón".to_string(), "pardon".to_string());
        
        // Basic responses
        m.insert("sí".to_string(), "yes".to_string());
        m.insert("no".to_string(), "no".to_string());
        m.insert("vale".to_string(), "okay".to_string());
        m.insert("tal vez".to_string(), "maybe".to_string());
        m.insert("quizás".to_string(), "perhaps".to_string());
        
        // Questions
        m.insert("¿cómo estás?".to_string(), "how are you".to_string());
        m.insert("qué".to_string(), "what".to_string());
        m.insert("dónde".to_string(), "where".to_string());
        m.insert("cuándo".to_string(), "when".to_string());
        m.insert("por qué".to_string(), "why".to_string());
        m.insert("quién".to_string(), "who".to_string());
        m.insert("cómo".to_string(), "how".to_string());
        
        // Common verbs
        m.insert("es".to_string(), "is".to_string());
        m.insert("son".to_string(), "are".to_string());
        m.insert("soy".to_string(), "am".to_string());
        m.insert("tengo".to_string(), "have".to_string());
        m.insert("tiene".to_string(), "has".to_string());
        m.insert("quiero".to_string(), "want".to_string());
        m.insert("necesito".to_string(), "need".to_string());
        m.insert("puedo".to_string(), "can".to_string());
        m.insert("voy a".to_string(), "will".to_string());
        m.insert("ir".to_string(), "go".to_string());
        m.insert("venir".to_string(), "come".to_string());
        m.insert("ver".to_string(), "see".to_string());
        m.insert("oír".to_string(), "hear".to_string());
        m.insert("hablar".to_string(), "speak".to_string());
        m.insert("decir".to_string(), "say".to_string());
        m.insert("saber".to_string(), "know".to_string());
        m.insert("pensar".to_string(), "think".to_string());
        m.insert("entender".to_string(), "understand".to_string());
        m.insert("ayudar".to_string(), "help".to_string());
        m.insert("ayúdame".to_string(), "help me".to_string());
        
        // Communication phrases
        m.insert("no entiendo".to_string(), "i don't understand".to_string());
        m.insert("habla despacio".to_string(), "speak slowly".to_string());
        m.insert("repite".to_string(), "repeat".to_string());
        m.insert("¿qué dijiste?".to_string(), "what did you say".to_string());
        m.insert("¿puedes repetir eso?".to_string(), "can you repeat that".to_string());
        m.insert("no hablo español".to_string(), "i don't speak spanish".to_string());
        m.insert("¿hablas inglés?".to_string(), "do you speak english".to_string());
        m.insert("¿cómo se dice?".to_string(), "how do you say".to_string());
        
        // Gaming/VOIP specific phrases
        m.insert("listo".to_string(), "ready".to_string());
        m.insert("espera".to_string(), "wait".to_string());
        m.insert("para".to_string(), "stop".to_string());
        m.insert("vamos".to_string(), "go".to_string());
        m.insert("bueno".to_string(), "good".to_string());
        m.insert("malo".to_string(), "bad".to_string());
        m.insert("excelente".to_string(), "great".to_string());
        m.insert("terrible".to_string(), "terrible".to_string());
        m.insert("ganar".to_string(), "win".to_string());
        m.insert("perder".to_string(), "lose".to_string());
        m.insert("juego".to_string(), "game".to_string());
        m.insert("jugar".to_string(), "play".to_string());
        m.insert("equipo".to_string(), "team".to_string());
        m.insert("jugador".to_string(), "player".to_string());
        m.insert("enemigo".to_string(), "enemy".to_string());
        m.insert("atacar".to_string(), "attack".to_string());
        m.insert("defender".to_string(), "defend".to_string());
        m.insert("mover".to_string(), "move".to_string());
        m.insert("correr".to_string(), "run".to_string());
        m.insert("saltar".to_string(), "jump".to_string());
        m.insert("disparar".to_string(), "shoot".to_string());
        m.insert("golpear".to_string(), "hit".to_string());
        m.insert("fallar".to_string(), "miss".to_string());
        m.insert("muerto".to_string(), "dead".to_string());
        m.insert("vivo".to_string(), "alive".to_string());
        m.insert("salud".to_string(), "health".to_string());
        m.insert("munición".to_string(), "ammo".to_string());
        m.insert("arma".to_string(), "weapon".to_string());
        m.insert("pistola".to_string(), "gun".to_string());
        m.insert("cuchillo".to_string(), "knife".to_string());
        m.insert("bomba".to_string(), "bomb".to_string());
        m.insert("granada".to_string(), "grenade".to_string());
        m.insert("cubrir".to_string(), "cover".to_string());
        m.insert("esconder".to_string(), "hide".to_string());
        m.insert("buscar".to_string(), "search".to_string());
        m.insert("encontrar".to_string(), "find".to_string());
        m.insert("mirar".to_string(), "look".to_string());
        m.insert("observar".to_string(), "watch".to_string());
        m.insert("escuchar".to_string(), "listen".to_string());
        m.insert("sonido".to_string(), "sound".to_string());
        m.insert("ruido".to_string(), "noise".to_string());
        m.insert("silencio".to_string(), "quiet".to_string());
        m.insert("fuerte".to_string(), "loud".to_string());
        
        std::sync::Mutex::new(m)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_translator_creation() {
        let translator = LocalTranslator::new();
        assert!(translator.is_ok());
    }

    #[tokio::test]
    async fn test_supported_languages() {
        let translator = LocalTranslator::new().unwrap();
        let pairs = translator.get_supported_language_pairs();
        assert!(!pairs.is_empty());
        
        // Check that English-Spanish is supported
        assert!(translator.is_supported("en", "es"));
        assert!(translator.is_supported("es", "en"));
    }

    #[tokio::test]
    async fn test_basic_translation() {
        let translator = LocalTranslator::new().unwrap();
        
        // Test English to Spanish
        let result = translator.translate("hello", "en", "es").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hola");
        
        // Test Spanish to English
        let result = translator.translate("hola", "es", "en").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }
} 