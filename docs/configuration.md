# Configuration

## Command Line Options
- `--config <file>`: Configuration file path (default: config.toml)
- `--debug`: Enable debug logging
- `--source-lang <lang>`: Source language code (default: en)
- `--target-lang <lang>`: Target language code (default: es)

## API Key Configuration

VoipGlot requires API keys for translation and text-to-speech services. Set these as environment variables:

### Required API Keys

#### DeepL Translation (Default)
```powershell
# Set DeepL API key
$env:DEEPL_API_KEY="your-deepl-api-key-here"
```

#### Azure Speech Services (Default TTS)
```powershell
# Set Azure Speech Services key
$env:AZURE_SPEECH_KEY="your-azure-speech-key-here"
$env:AZURE_REGION="eastus"  # or your preferred region
```

### Optional API Keys

#### Google APIs
```powershell
$env:GOOGLE_API_KEY="your-google-api-key-here"
```

#### ElevenLabs TTS
```powershell
$env:ELEVENLABS_API_KEY="your-elevenlabs-api-key-here"
```

### Setting Environment Variables Permanently

#### For Current Session Only
```powershell
$env:DEEPL_API_KEY="your-key"
$env:AZURE_SPEECH_KEY="your-key"
$env:AZURE_REGION="eastus"
```

#### For Permanent Setup (Windows)
```powershell
# Set system environment variables
[Environment]::SetEnvironmentVariable("DEEPL_API_KEY", "your-key", "User")
[Environment]::SetEnvironmentVariable("AZURE_SPEECH_KEY", "your-key", "User")
[Environment]::SetEnvironmentVariable("AZURE_REGION", "eastus", "User")
```

### Getting API Keys

1. **DeepL**: Sign up at [DeepL API](https://www.deepl.com/pro-api) (free tier available)
2. **Azure Speech Services**: Create a resource in [Azure Portal](https://portal.azure.com)
3. **Google Cloud**: Enable Speech-to-Text and Translate APIs in [Google Cloud Console](https://console.cloud.google.com)
4. **ElevenLabs**: Sign up at [ElevenLabs](https://elevenlabs.io) (free tier available)

## Config File
Edit `config.toml` to customize:
- Audio devices and settings
- Translation providers
- Processing parameters
- API endpoints

## Provider Configuration

You can change providers in `config.toml`:

```toml
[translation]
stt_provider = "Whisper"        # Whisper, Azure, Google
translation_provider = "DeepL"  # DeepL, Google, Azure
tts_provider = "Azure"          # Azure, ElevenLabs, Google
``` 