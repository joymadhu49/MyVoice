# MyVoice — Session Summary (2026-05-06)

## Goal
Clone of `BridgeVoice.AppImage` (privacy-first voice dictation) for Linux.

## Reverse-engineered original
- Source: `~/Downloads/BridgeVoice.AppImage` (extracted to `~/Downloads/squashfs-root/`)
- Stack: **Tauri (Rust) + whisper.cpp (ggml)**
- Frontend: GTK webview
- Model dir empty in AppImage → downloaded at runtime
- Plugins: `tauri_plugin_http`, sentry telemetry

## Built: `~/MyVoice`
Tauri 2 + vanilla HTML/JS frontend + Rust backend with whisper-rs and cpal.

### Key files
- `src-tauri/Cargo.toml` — deps: tauri 2, cpal 0.15, whisper-rs 0.13, hound, dirs, ureq, anyhow
- `src-tauri/src/lib.rs` — commands: `start_recording`, `stop_recording`. Uses cpal default input device, supports F32/I16/U16 sample formats, mono mix + linear-resample to 16kHz, runs whisper greedy decode
- `src/index.html` + `src/main.js` — single button toggle UI
- Model auto-downloads to `~/.local/share/myvoice/ggml-base.en.bin` (~148MB) on first transcribe
- Model URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin`

## Toolchain installed
- Rust 1.95.0 via rustup (`~/.cargo/bin`) — `source ~/.cargo/env` to use
- Node 18.19.1 + npm 9.2.0 (apt)
- System deps: `libwebkit2gtk-4.1-dev`, `build-essential`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libasound2-dev`, `pkg-config`, `cmake`, `clang`

## Status
- ✅ Compiles clean (`cargo check` passed)
- ✅ Dev binary runs, window opens
- ✅ Model downloaded (full 148MB)
- ✅ Whisper transcription verified (saw tokens streaming in logs)
- ⏳ End-to-end transcription quality not yet validated by user

## Critical gotcha: snap env pollution
Claude Code runs from VS Code **snap** (`/snap/code/237/...`). Spawning `myvoice` binary from VS Code's terminal loads snap's `libpthread.so.0` → `__libc_pthread_init: GLIBC_PRIVATE` symbol error.

**Workaround**: launch with clean env:
```bash
cd ~/MyVoice && env -i HOME="$HOME" \
  PATH="/home/joy/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
  DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" \
  WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
  XDG_RUNTIME_DIR="/run/user/1000" \
  DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
  USER="$USER" \
  npm run tauri dev
```

**Better**: run from a non-snap terminal (gnome-terminal launched outside VS Code).

## Run / build commands
- Dev: `cd ~/MyVoice && npm run tauri dev` (use clean-env wrapper above if needed)
- Release AppImage + .deb: `cd ~/MyVoice && npm run tauri build`
  - Output: `src-tauri/target/release/bundle/{appimage,deb}/`

## Known limitations / next steps
- English-only model. Swap URL to `ggml-base.bin` for multilang
- No global hotkey, no autopaste-into-active-window (BridgeVoice has these — not yet ported)
- No system-tray icon
- Linear resampler is crude — could swap for `rubato` for quality
- No mic-permission UX, no device selector
- Audio thread uses `thread::sleep` polling — fine but inelegant; could use channel
- App icon = default Tauri (replace icons in `src-tauri/icons/`)
- Identifier `com.myvoice.app` in `tauri.conf.json` — change before publishing
- No tests

## How to resume in new session
1. Read this file
2. `cd ~/MyVoice`
3. Check whether dev still runs: `pgrep -af myvoice`
4. If not, relaunch with the clean-env command above
5. Pick next item from "Known limitations" or ask user priorities
