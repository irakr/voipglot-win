# Troubleshooting

## Common Issues

### Audio Device Not Found
- Ensure your microphone is properly connected and recognized by Windows
- Check Windows Sound settings
- Try running with `--debug` to see available devices

### High Latency
- Reduce `chunk_duration_ms` in config.toml
- Use lower latency audio devices
- Enable hardware acceleration if available

### Translation Errors
- Verify API keys are correctly set
- Check internet connectivity
- Ensure language codes are supported by your chosen provider

### Virtual Microphone Not Working
- Verify VB-CABLE is properly installed
- Check that VB-CABLE is selected as output device
- Restart target applications after changing audio settings

### Debug Mode
Run with `--debug` flag to get detailed logging:
```powershell
./target/x86_64-pc-windows-msvc/fast-release/voipglot-win.exe --debug
``` 