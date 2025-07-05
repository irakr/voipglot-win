# VoipGlot Windows

Real-time audio translation for Windows gaming and VOIP applications.

## Quickstart

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd voipglot-win
   ```
2. **Install Rust and dependencies:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add x86_64-pc-windows-msvc
   ```
3. **Configure API keys:**
   See [docs/configuration.md](docs/configuration.md)
4. **Build the application:**
   ```powershell
   # Fast development build
   ./build-windows.ps1 --fast
   # Production build
   ./build-windows.ps1
   ```
5. **Run the application:**
   ```powershell
   ./target/x86_64-pc-windows-msvc/fast-release/voipglot-win.exe
   # Or for production:
   ./target/x86_64-pc-windows-msvc/release/voipglot-win.exe
   ```

## Documentation

- [Features](docs/features.md)
- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Usage & Optimization](docs/usage.md)
- [AI Providers & Supported Languages](docs/providers.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Performance](docs/performance.md)
- [Roadmap](docs/roadmap.md)

For more details, see the documentation in the `docs/` directory.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 