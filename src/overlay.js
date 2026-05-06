const { listen } = window.__TAURI__.event;

const hud = document.getElementById("hud");

function setState(s) {
  hud.dataset.state = s;
}

setState("idle");

listen("rec-state", (e) => {
  const s = e.payload;
  if (s === "recording" || s === "transcribing" || s === "done" || s === "idle") {
    setState(s);
  }
});
