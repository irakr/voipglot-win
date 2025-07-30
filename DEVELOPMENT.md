# VoipGlot Windows Development Guide

## Quick Start

### Prerequisites
- Rust 1.88.0 (automatically managed by rust-toolchain.toml)
- Node.js and npm
- Visual Studio Build Tools

### Development Workflow

**✅ CORRECT WAY (Automated):**
```powershell
cd voipglot-win
.\build.ps1 -TauriDev
```

This will:
- **Automatically build the frontend** (npm run build)
- **Generate fresh dist/ directory** with latest TypeScript changes
- **Open the native Tauri window** with full functionality
- **Automatically open Developer Tools** for debugging
- **Enable hot reload** for frontend development
- **Provide access to all Tauri backend features**

**✅ For Frontend-Only Changes:**
```powershell
cd voipglot-win
.\build.ps1 -FrontendBuild
```

This will:
- **Build only the frontend** (npm run build)
- **Update the dist/ directory** with latest changes
- **Skip Tauri backend** (faster for frontend-only work)

**❌ WRONG WAY (Will Show Error):**
```powershell
# Don't do this - it will show an error overlay
npm run dev
# Then accessing http://localhost:1420/ in browser
```

**🔄 For Frontend Changes During Development:**
1. **Make changes** to TypeScript files in `src/`
2. **Rebuild frontend**: `.\build.ps1 -FrontendBuild`
3. **Restart Tauri**: `cargo tauri dev` (or use `.\build.ps1 -TauriDev` again)

## Debugging

### Frontend Logs (TypeScript/UI)
When you run `cargo tauri dev`:
- Developer Tools automatically opens
- Console tab shows all TypeScript logs
- Real-time logging of UI interactions
- Hot reload for instant feedback

### Backend Logs (Rust/Backend)
- **File**: `voipglot-win.log` in the application directory
- **Real-time monitoring**: `Get-Content voipglot-win.log -Wait`
- **Last 50 lines**: `Get-Content voipglot-win.log -Tail 50`

### Combined Debugging
1. Run `cargo tauri dev`
2. Keep both frontend console and backend log file open
3. Test UI interactions and watch both logs simultaneously

## Project Structure

```
voipglot-win/
├── src-tauri/          # Tauri backend (Rust)
│   ├── src/
│   │   ├── main.rs     # Tauri application entry point
│   │   └── lib.rs      # Tauri backend library with commands
│   ├── Cargo.toml      # Rust dependencies
│   └── tauri.conf.json # Tauri application configuration
├── src/                # Frontend source (HTML/TS/CSS)
│   ├── index.html      # Frontend entry point
│   ├── main.ts         # TypeScript frontend logic
│   └── styles.css      # Frontend styling
├── package.json        # Frontend dependencies
├── vite.config.ts      # Vite build configuration
└── rust-toolchain.toml # Rust version specification (1.88.0)
```

## Common Issues

### "Tauri Not Available" Error
- **Cause**: Accessing `http://localhost:1420/` directly in browser
- **Solution**: Use `cargo tauri dev` instead

### Frontend Changes Not Reflecting
- **Cause**: Running `npm run dev` instead of `cargo tauri dev`
- **Solution**: Use `cargo tauri dev` for hot reload

### Backend Changes Not Reflecting
- **Cause**: Need to restart Tauri development server
- **Solution**: Stop `cargo tauri dev` and restart it

## Build Commands

```powershell
# Development (with hot reload)
cargo tauri dev

# Production build
cargo tauri build

# Frontend only build (for testing)
npm run build
```

## Rust Version

The project uses Rust 1.88.0 as specified in `rust-toolchain.toml`. This ensures consistent builds across different environments. 