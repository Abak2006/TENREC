# TENREC Development Guide

## Prerequisites
- Node.js LTS (v24+) & npm (v11+)
- Rust (v1.98+) with `x86_64-pc-windows-gnu` or `x86_64-pc-windows-msvc` toolchain
- MSYS2 / GCC 15+ toolchain (UCRT64)
- Microsoft Edge WebView2 runtime

## Project Structure
- `src/`: React + TypeScript frontend (Vite, modern dark design system, dense layout)
- `src-tauri/`: Rust native backend (real Windows telemetry, correlation, detection, approval, restoration, persistence)

## Common Commands
```powershell
# Install frontend dependencies
npm install

# Run frontend build
npm run build

# Run Rust tests
cd src-tauri
cargo test

# Launch Tauri development mode
npm run tauri dev

# Package production installer
npm run tauri build
```
