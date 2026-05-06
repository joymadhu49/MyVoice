# MyVoice

Privacy-first voice dictation. Local Whisper inference. Global hotkey + auto-type.

Tauri 2 + Rust + whisper.cpp + cpal + enigo.

## Hotkey

`Ctrl+Alt+Space` anywhere → start/stop recording. Transcript auto-types into focused field + clipboard.

## Build

### Linux (Ubuntu/Debian)

```bash
sudo apt install -y build-essential curl wget file pkg-config cmake clang \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev \
  librsvg2-dev libasound2-dev nodejs npm
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
npm install
npm run tauri build
# output: src-tauri/target/release/bundle/{appimage,deb}/
```

Run dev: `npm run tauri dev`

### macOS

```bash
xcode-select --install
brew install node cmake
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
npm install
npm run tauri build
# output: src-tauri/target/release/bundle/{dmg,macos}/
```

### Windows

WebView2 + Visual Studio Build Tools + Rust + Node, then `npm run tauri build`.

## Model

Auto-downloads `ggml-base.en.bin` (~148MB) on first transcribe to:
- Linux: `~/.local/share/myvoice/`
- macOS: `~/Library/Application Support/myvoice/`
- Windows: `%APPDATA%\myvoice\`

Multilang: change `MODEL_URL` in `src-tauri/src/lib.rs` to `ggml-base.bin` and adjust `set_language` arg.

## Snap-env gotcha (Linux only)

Running from VS Code's snap terminal poisons env (`GTK_PATH`, `LOCPATH`) → `libpthread.so.0: undefined symbol`. Fix: launch from a non-snap terminal, or:

```bash
env -i HOME="$HOME" PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
  DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" \
  WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="/run/user/$(id -u)" \
  DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" USER="$USER" \
  npm run tauri dev
```
