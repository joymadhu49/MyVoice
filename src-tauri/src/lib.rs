use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use enigo::{Enigo, Keyboard, Settings};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";

struct Session {
    stop: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
    handle: Option<thread::JoinHandle<()>>,
}

#[derive(Default)]
struct AppState {
    session: Mutex<Option<Session>>,
    whisper: Mutex<Option<WhisperContext>>,
}

fn model_path() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| anyhow!("no data dir"))?
        .join("myvoice");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("ggml-base.en.bin"))
}

fn ensure_model() -> Result<PathBuf> {
    let p = model_path()?;
    if p.exists() && fs::metadata(&p)?.len() > 140_000_000 {
        return Ok(p);
    }
    let resp = ureq::get(MODEL_URL).call()?;
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(&p)?;
    std::io::copy(&mut reader, &mut file)?;
    Ok(p)
}

fn start_inner(state: &AppState) -> Result<(), String> {
    let mut sess = state.session.lock().unwrap();
    if sess.is_some() {
        return Err("already recording".into());
    }
    let stop = Arc::new(AtomicBool::new(false));
    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let sr = Arc::new(Mutex::new(0u32));
    let ch = Arc::new(Mutex::new(0u16));

    let stop_t = stop.clone();
    let samples_t = samples.clone();
    let sr_t = sr.clone();
    let ch_t = ch.clone();

    let handle = thread::spawn(move || {
        let host = cpal::default_host();
        let dev = match host.default_input_device() {
            Some(d) => d,
            None => return,
        };
        let cfg = match dev.default_input_config() {
            Ok(c) => c,
            Err(_) => return,
        };
        *sr_t.lock().unwrap() = cfg.sample_rate().0;
        *ch_t.lock().unwrap() = cfg.channels();
        let fmt = cfg.sample_format();
        let cfg2: cpal::StreamConfig = cfg.into();
        let s2 = samples_t.clone();
        let err_fn = |e| eprintln!("audio err: {}", e);
        let stream = match fmt {
            SampleFormat::F32 => dev.build_input_stream(
                &cfg2,
                move |data: &[f32], _: &_| s2.lock().unwrap().extend_from_slice(data),
                err_fn,
                None,
            ),
            SampleFormat::I16 => dev.build_input_stream(
                &cfg2,
                move |data: &[i16], _: &_| {
                    let mut g = s2.lock().unwrap();
                    g.extend(data.iter().map(|&v| v as f32 / 32768.0));
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => dev.build_input_stream(
                &cfg2,
                move |data: &[u16], _: &_| {
                    let mut g = s2.lock().unwrap();
                    g.extend(data.iter().map(|&v| (v as f32 - 32768.0) / 32768.0));
                },
                err_fn,
                None,
            ),
            _ => return,
        };
        let stream = match stream {
            Ok(s) => s,
            Err(_) => return,
        };
        if stream.play().is_err() {
            return;
        }
        while !stop_t.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }
        drop(stream);
    });

    *sess = Some(Session {
        stop,
        samples,
        sample_rate: sr,
        channels: ch,
        handle: Some(handle),
    });
    Ok(())
}

fn to_mono_16k(input: &[f32], sample_rate: u32, channels: u16) -> Vec<f32> {
    let mono: Vec<f32> = if channels <= 1 {
        input.to_vec()
    } else {
        input
            .chunks(channels as usize)
            .map(|c| c.iter().sum::<f32>() / c.len() as f32)
            .collect()
    };
    if sample_rate == 16000 || mono.is_empty() {
        return mono;
    }
    let ratio = 16000.0 / sample_rate as f32;
    let out_len = (mono.len() as f32 * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f32 / ratio;
        let i0 = src.floor() as usize;
        let i1 = (i0 + 1).min(mono.len() - 1);
        let t = src - i0 as f32;
        out.push(mono[i0] * (1.0 - t) + mono[i1] * t);
    }
    out
}

fn stop_inner(state: &AppState) -> Result<String, String> {
    let sess = {
        let mut g = state.session.lock().unwrap();
        g.take()
    };
    let mut sess = sess.ok_or_else(|| "not recording".to_string())?;
    sess.stop.store(true, Ordering::Relaxed);
    if let Some(h) = sess.handle.take() {
        let _ = h.join();
    }
    let raw = sess.samples.lock().unwrap().clone();
    let sr = *sess.sample_rate.lock().unwrap();
    let ch = *sess.channels.lock().unwrap();
    if raw.is_empty() {
        return Err("no audio captured".into());
    }
    let pcm = to_mono_16k(&raw, sr, ch);

    let model = ensure_model().map_err(|e| format!("model: {}", e))?;

    let mut wlock = state.whisper.lock().unwrap();
    if wlock.is_none() {
        let ctx = WhisperContext::new_with_params(
            model.to_str().unwrap(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("whisper init: {}", e))?;
        *wlock = Some(ctx);
    }
    let ctx = wlock.as_ref().unwrap();
    let mut state_w = ctx.create_state().map_err(|e| e.to_string())?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_language(Some("en"));
    state_w.full(params, &pcm).map_err(|e| e.to_string())?;
    let n = state_w.full_n_segments().map_err(|e| e.to_string())?;
    let mut out = String::new();
    for i in 0..n {
        if let Ok(seg) = state_w.full_get_segment_text(i) {
            out.push_str(&seg);
        }
    }
    Ok(out.trim().to_string())
}

fn deliver_text(text: &str) {
    if text.is_empty() {
        return;
    }
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text.to_string());
    }
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.text(text);
    }
}

#[tauri::command]
fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    start_inner(&state)
}

#[tauri::command]
async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    let text = stop_inner(&state)?;
    let t = text.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        deliver_text(&t);
    });
    Ok(text)
}

fn toggle_via_hotkey(app: &AppHandle) {
    let state = app.state::<AppState>();
    let recording = state.session.lock().unwrap().is_some();
    let app2 = app.clone();
    if !recording {
        let _ = start_inner(&state);
        let _ = app.emit("rec-state", "recording");
    } else {
        std::thread::spawn(move || {
            let st = app2.state::<AppState>();
            match stop_inner(&st) {
                Ok(text) => {
                    let _ = app2.emit("transcript", &text);
                    let _ = app2.emit("rec-state", "idle");
                    std::thread::sleep(Duration::from_millis(300));
                    deliver_text(&text);
                }
                Err(e) => {
                    let _ = app2.emit("rec-error", e);
                    let _ = app2.emit("rec-state", "idle");
                }
            }
        });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::Space)
                    {
                        toggle_via_hotkey(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let sc = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
            app.global_shortcut().register(sc)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
