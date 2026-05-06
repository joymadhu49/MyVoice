# MyVoice

Privacy-first voice dictation. Push-to-talk hotkey. Local Whisper or Groq Cloud. Auto-paste into focused window. History + voice profile.

Tauri 2 + Rust + whisper.cpp + cpal + enigo + Groq Whisper API.

## Features

- **Push-to-talk hotkey:** hold `Ctrl + Shift + Space`, speak, release. Transcript auto-pastes into focused field + clipboard.
- **Floating HUD overlay:** frameless pill bottom-center shows recording / transcribing / done states.
- **Two providers:**
  - **Local Whisper** (offline, private). Tiny / Base / Small × English-only / multilingual. Auto-download.
  - **Groq Cloud** (online, fastest + most accurate). `whisper-large-v3-turbo` / `whisper-large-v3` / `distil-whisper-large-v3-en`.
- **Voice profile:** top-N words from your history are sent as a Whisper context prompt — biases recognition toward your jargon, names, acronyms.
- **History:** every dictation persisted (`history.jsonl`). Per-row copy / flag / delete. Grouped by Today / Yesterday / weekday / date.
- **Stats:** total words, WPM (avg), day streak, sessions.
- **Audio quality:** peak normalize + silence trim + beam search (5) + `suppress_blank` + `no_context`.

## Hotkey

`Ctrl + Shift + Space` anywhere → hold to record, release to transcribe + auto-paste.

## Build

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

Run dev: `npm run tauri dev`

Permissions on first run: grant **Microphone** + **Accessibility** (System Settings → Privacy & Security). Without Accessibility, transcript still hits the clipboard but won't auto-type.

Unsigned build: right-click `.app` → Open. Or `xattr -dr com.apple.quarantine /Applications/myvoice.app`.

### Linux (Ubuntu/Debian)

```bash
sudo apt install -y build-essential curl wget file pkg-config cmake clang \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev \
  librsvg2-dev libasound2-dev nodejs npm
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
git clone https://github.com/joymadhu49/MyVoice.git
cd MyVoice
npm install
npm run tauri build
# output: src-tauri/target/release/bundle/{appimage,deb}/
```

Run dev: `npm run tauri dev`

#### Snap-env gotcha

Running from VS Code's snap terminal poisons env (`GTK_PATH`, `LOCPATH`) → `libpthread.so.0: undefined symbol`. Fix: launch from a non-snap terminal, or:

```bash
env -i HOME="$HOME" PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
  DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" \
  WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="/run/user/$(id -u)" \
  DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" USER="$USER" \
  npm run tauri dev
```

#### Linux notes

- Auto-paste uses `xdotool` (X11) via `enigo`. On Wayland, you may need to enable XWayland or rely on the clipboard fallback.
- Title bar is shown (macOS-only Overlay style is ignored on Linux).

### Windows

WebView2 + Visual Studio Build Tools + Rust + Node, then `npm run tauri build`.

## Cloud (Groq) setup

1. Get a free API key at [console.groq.com/keys](https://console.groq.com/keys).
2. Settings tab → Provider → Groq Cloud.
3. Paste key. Click "Test key". Pick model (`whisper-large-v3-turbo` recommended).

API key is stored locally in `settings.json` in the data dir below.

## Data locations

| OS | Path |
|---|---|
| macOS | `~/Library/Application Support/myvoice/` |
| Linux | `~/.local/share/myvoice/` |
| Windows | `%APPDATA%\myvoice\` |

Files:
- `settings.json` — provider, language, hotkey, Groq key
- `history.jsonl` — append-only dictation log
- `ggml-*.bin` — downloaded Whisper models

## Models (local)

| Model | Size | Notes |
|---|---|---|
| `tiny.en` | 75 MB | Fastest, English-only |
| `base.en` | 142 MB | Default, balanced |
| `small.en` | 466 MB | Most accurate, English-only |
| `tiny` / `base` / `small` | same | Multilingual variants |

## Architecture

- **`src-tauri/src/lib.rs`** — Rust backend. Audio capture (cpal), resampling to 16k mono, peak normalize, silence trim, Whisper inference (whisper-rs) or Groq HTTP multipart, history persistence, voice profile prompt builder, global hotkey, push-to-talk, HUD show/hide, model download with progress events.
- **`src/index.html` + `main.js`** — single-window UI: sidebar nav, home (history), stats, voice profile, settings tabs.
- **`src/overlay.html` + `overlay.js`** — frameless transparent always-on-top HUD pill.
- **`src-tauri/tauri.conf.json`** — windows config, macOS Overlay titleBarStyle.

## License

MIT
