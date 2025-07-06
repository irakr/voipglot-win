# Local Translation Engine

VoipGlot now supports **local translation** using MarianMT models, providing offline translation capabilities without requiring API keys or internet connectivity.

## Features

- **Offline Operation**: No internet connection required
- **No API Keys**: Completely free to use
- **Fast Inference**: Optimized for real-time translation
- **High Quality**: Based on state-of-the-art MarianMT models
- **Model Caching**: Models are downloaded once and cached locally

## Supported Language Pairs

The local translator supports the following language pairs:

### English-Based Pairs
- English ↔ Spanish (en-es, es-en)
- English ↔ French (en-fr, fr-en)
- English ↔ German (en-de, de-en)
- English ↔ Italian (en-it, it-en)
- English ↔ Portuguese (en-pt, pt-en)
- English ↔ Russian (en-ru, ru-en)
- English ↔ Japanese (en-ja, ja-en)
- English ↔ Korean (en-ko, ko-en)
- English ↔ Chinese (en-zh, zh-en)
- English ↔ Arabic (en-ar, ar-en)
- English ↔ Hindi (en-hi, hi-en)

### Cross-Language Pairs
- Spanish ↔ French (es-fr, fr-es)
- German ↔ French (de-fr, fr-de)

## Configuration

### Enable Local Translation

Set the translation provider to "Local" in your `config.toml`:

```toml
[translation]
translation_provider = "Local"  # Use local MarianMT models
```

### Model Cache Directory

Models are automatically downloaded and cached. You can customize the cache location:

```bash
# Set custom cache directory (optional)
export VOIPGLOT_MODEL_CACHE="C:\Users\YourName\.voipglot\models"

# Default location: ~/.voipglot/models (Linux/Mac) or %USERPROFILE%\.voipglot\models (Windows)
```

## Usage

### First Run

On first use, the local translator will:

1. **Download Models**: Automatically download the required MarianMT model (~500MB-1GB)
2. **Cache Models**: Store models locally for future use
3. **Initialize Pipeline**: Load the model into memory

```bash
# First run - will download models
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe --source-lang en --target-lang es
```

### Subsequent Runs

After the initial download, models load from cache:

```bash
# Fast startup - models loaded from cache
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe --source-lang en --target-lang es
```

## Performance

### Model Sizes
- **Small models**: ~500MB (most language pairs)
- **Large models**: ~1GB (complex language pairs)

### Memory Usage
- **Loaded model**: ~2-4GB RAM (depending on model size)
- **Inference speed**: ~100-500ms per sentence

### Optimization Tips

1. **Use Fast Build**: For development, use `--fast` build profile
2. **Model Preloading**: Models are cached after first use
3. **Memory Management**: Close other applications if using large models

## Comparison with Cloud Providers

| Feature | Local (MarianMT) | DeepL | Google | Azure |
|---------|------------------|-------|--------|-------|
| **Internet Required** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **API Key Required** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **Cost** | 💰 Free | 💰 Paid | 💰 Paid | 💰 Paid |
| **Speed** | ⚡ Fast | ⚡ Fast | ⚡ Fast | ⚡ Fast |
| **Quality** | 🎯 Good | 🎯 Excellent | 🎯 Good | 🎯 Good |
| **Language Support** | 🌍 Limited | 🌍 Wide | 🌍 Wide | 🌍 Wide |
| **Privacy** | 🔒 Full | 🔒 Partial | 🔒 Partial | 🔒 Partial |

## Troubleshooting

### Model Download Issues

If model download fails:

```bash
# Check internet connection
ping google.com

# Clear cache and retry
rm -rf ~/.voipglot/models
# or on Windows:
rmdir /s %USERPROFILE%\.voipglot\models
```

### Memory Issues

If you encounter memory errors:

1. **Close other applications** to free RAM
2. **Use smaller models** for less common language pairs
3. **Restart the application** to clear memory

### Unsupported Language Pairs

If your language pair isn't supported:

1. **Check supported pairs**: See list above
2. **Use cloud provider**: Switch to DeepL, Google, or Azure
3. **Request addition**: Open an issue for new language pairs

## Advanced Configuration

### Custom Model Paths

You can specify custom model paths (advanced users):

```bash
# Set custom model cache
export VOIPGLOT_MODEL_CACHE="/path/to/custom/models"
```

### Model Management

List and manage cached models:

```bash
# View cached models
ls ~/.voipglot/models

# Remove specific model
rm -rf ~/.voipglot/models/Helsinki-NLP_opus-mt-en-es
```

## Development

### Adding New Language Pairs

To add support for new language pairs:

1. **Find MarianMT model**: Check [Hugging Face MarianMT models](https://huggingface.co/models?pipeline_tag=translation&sort=downloads)
2. **Add to mapping**: Update `MARIAN_MODELS` in `local_translator.rs`
3. **Test**: Verify the model works correctly

### Example: Adding French-German

```rust
// In local_translator.rs
m.insert("fr-de".to_string(), "Helsinki-NLP/opus-mt-fr-de".to_string());
m.insert("de-fr".to_string(), "Helsinki-NLP/opus-mt-de-fr".to_string());
```

## Benefits

### Privacy
- **No data sent to cloud**: All processing happens locally
- **No logging**: No translation requests logged externally
- **Complete control**: You own your data

### Cost
- **Free forever**: No API costs or usage limits
- **No rate limits**: Translate as much as you want
- **No subscriptions**: One-time setup

### Reliability
- **No internet dependency**: Works offline
- **No API downtime**: No external service failures
- **Consistent performance**: Predictable response times

## Limitations

### Language Support
- **Limited pairs**: Only ~25 language pairs vs 100+ for cloud providers
- **Quality variation**: Some pairs may have lower quality than cloud services

### Resource Usage
- **Memory intensive**: Requires 2-4GB RAM per loaded model
- **Storage**: Models take 500MB-1GB disk space each
- **Initial download**: First use requires downloading models

### Performance
- **Cold start**: First translation may be slower
- **Model loading**: Takes time to load models into memory 