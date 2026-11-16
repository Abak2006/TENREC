# Development

## Prerequisites

- Node.js 24+ and npm
- Rust 1.77+ with the MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`)
- Microsoft Edge WebView2 runtime (included in Windows 11)

## Layout

- `src/`: React 19 + TypeScript UI. Tailwind 4 with the shadcn/Radix components in `src/components/ui`.
  - `lib/types.ts` mirrors the Rust serde types; keep them in sync.
  - `lib/store.ts` polls the backend.
  - `pages/` has one file per screen.
- `src-tauri/src/`: Rust core. See [ARCHITECTURE.md](ARCHITECTURE.md).
- `src-tauri/tests/`: integration tests against real Windows APIs. The library has no inline unit tests: its unit-test harness cannot link the app manifest and fails to start on Windows, so `[lib] test = false`.

## Commands

```bash
npm install
npm run tauri dev           # Vite on http://localhost:5173 + the native app
npm run build               # tsc + vite production build
npm run lint
cd src-tauri && cargo test && cargo clippy --all-targets
npm run tauri build         # MSI + NSIS installers
```

Build release binaries with `npm run tauri build` (add `-- --no-bundle` to skip the installers), not plain `cargo build --release`. The Tauri CLI enables the feature that embeds the UI; without it the binary tries to load the dev server at `localhost:5173` and shows "can't reach this page".

To test firewall actions, start a terminal as administrator and run `npm run tauri dev` from it.
