# Vibe Coding Progress Bar

A floating progress bar for AI-assisted coding, built with Tauri and React.

## Prerequisites

- **Rust** (1.70.0 or later): https://rustup.rs/
- **Node.js** (18.0 or later): https://nodejs.org/
- **pnpm** (recommended) or npm

## Installation

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Node.js (if not installed):
   ```bash
   # Using nvm (recommended)
   nvm install 20
   nvm use 20
   
   # Or download from https://nodejs.org/
   ```

3. Install dependencies:
   ```bash
   npm install
   ```

4. Install Tauri CLI:
   ```bash
   npm install -D @tauri/cli
   ```

## Development

Start development server:
```bash
npm run tauri dev
```

## Build

Build for current platform:
```bash
npm run tauri build
```

Build for all platforms:
```bash
npm run tauri build -- --target universal-apple-darwin  # macOS
npm run tauri build --target x86_64-unknown-linux-gnu   # Linux
npm run tauri build --target x86_64-pc-windows-msvc     # Windows
```

## Project Structure

```
vibeProcessBar/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── stores/             # Zustand state management
│   ├── hooks/              # Custom React hooks
│   ├── utils/              # Utility functions
│   └── App.tsx             # Main app component
├── src-tauri/              # Rust backend
│   ├── src/main.rs         # Tauri entry point
│   ├── tauri.conf.json     # Tauri configuration
│   └── Cargo.toml          # Rust dependencies
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Features

- Floating window with transparent background
- Circular progress indicator
- Drag to reposition
- Double-click to reset
- Zustand state management with persistence
- Theme support
- Window position persistence

## License

MIT
