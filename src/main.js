const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let recording = false;
let btn, out, status;

function setRecording(on) {
  recording = on;
  btn.textContent = on ? "Stop & transcribe" : "Start recording";
  btn.classList.toggle("on", on);
}

async function toggle() {
  if (!recording) {
    try {
      await invoke("start_recording");
      setRecording(true);
      status.textContent = "Recording... (Ctrl+Alt+Space to stop)";
    } catch (e) {
      status.textContent = "Error: " + e;
    }
  } else {
    btn.disabled = true;
    status.textContent = "Transcribing...";
    try {
      const text = await invoke("stop_recording");
      out.textContent = (out.textContent + "\n" + text).trim();
      status.textContent = "Done. Pasted into focused window + clipboard.";
    } catch (e) {
      status.textContent = "Error: " + e;
    }
    btn.disabled = false;
    setRecording(false);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  btn = document.querySelector("#rec");
  out = document.querySelector("#out");
  status = document.querySelector("#status");
  btn.addEventListener("click", toggle);

  await listen("rec-state", (e) => {
    setRecording(e.payload === "recording");
    status.textContent = e.payload === "recording"
      ? "Recording... (Ctrl+Alt+Space to stop)"
      : "Transcribing...";
  });
  await listen("transcript", (e) => {
    out.textContent = (out.textContent + "\n" + e.payload).trim();
    status.textContent = "Done. Pasted + clipboard.";
  });
  await listen("rec-error", (e) => {
    status.textContent = "Error: " + e.payload;
  });
});
