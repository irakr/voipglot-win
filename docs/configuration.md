# Configuration

## Command Line Options
- `--config <file>`: Configuration file path (default: config.toml)
- `--debug`: Enable debug logging
- `--source-lang <lang>`: Source language code (default: en)
- `--target-lang <lang>`: Target language code (default: es)

## Config File
Edit `config.toml` to customize:
- Audio devices and settings
- Translation settings (local only)
- Processing parameters

## Example Config

```toml
[audio]
input_device = ""
output_device = ""
sample_rate = 48000
channels = 2
buffer_size = 1024
latency_ms = 50

[translation]
source_language = "en"
target_language = "es"

[processing]
chunk_duration_ms = 1000
silence_threshold = 0.01
noise_reduction = true
echo_cancellation = true
```

## Provider

- Only the local translation engine (MarianMT) is supported.
- No API keys or internet required. 