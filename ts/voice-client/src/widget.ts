import { AudioCapture } from "@lx/audio-capture";
import { AudioPlayback } from "@lx/audio-playback";
import { registerWidget } from "@lx/widget-bridge/src/registry";
import type { Widget } from "@lx/widget-bridge/src/registry";
import type { Dioxus } from "@lx/widget-bridge/src/types";

type VoiceStatus = "idle" | "listening" | "processing" | "speaking";

interface VoiceState {
  capture: AudioCapture;
  playback: AudioPlayback;
  status: VoiceStatus;
  statusEl: HTMLDivElement;
  transcriptEl: HTMLDivElement;
  dx: Dioxus;
}

const states = new Map<string, VoiceState>();

function setStatus(state: VoiceState, status: VoiceStatus): void {
  state.status = status;
  const labels: Record<VoiceStatus, string> = {
    idle: "Idle",
    listening: "Listening...",
    processing: "Processing...",
    speaking: "Speaking...",
  };
  state.statusEl.textContent = labels[status];
  state.dx.send({ type: "status_change", status });
}

function addEntry(
  transcriptEl: HTMLDivElement,
  cls: string,
  text: string
): void {
  const div = document.createElement("div");
  div.style.padding = "4px 0";
  div.style.borderBottom = "1px solid #2a2a4e";
  div.style.fontSize = "14px";
  if (cls === "you") div.style.color = "#64b5f6";
  else if (cls === "agent") div.style.color = "#81c784";
  else if (cls === "error") div.style.color = "#ef5350";
  div.textContent = text;
  transcriptEl.appendChild(div);
  transcriptEl.scrollTop = transcriptEl.scrollHeight;
}

const voiceWidget: Widget = {
  mount(elementId: string, _config: unknown, dx: Dioxus) {
    const el = document.getElementById(elementId);
    if (!el) return;

    const container = document.createElement("div");
    container.style.display = "flex";
    container.style.flexDirection = "column";
    container.style.height = "100%";

    const statusEl = document.createElement("div");
    statusEl.style.padding = "12px 16px";
    statusEl.style.fontSize = "16px";
    statusEl.style.fontWeight = "600";
    statusEl.style.borderBottom = "1px solid #333";
    statusEl.style.color = "#e0e0e0";
    statusEl.textContent = "Idle";

    const transcriptEl = document.createElement("div");
    transcriptEl.style.flex = "1";
    transcriptEl.style.overflowY = "auto";
    transcriptEl.style.padding = "12px 16px";

    const btnRow = document.createElement("div");
    btnRow.style.display = "flex";
    btnRow.style.gap = "8px";
    btnRow.style.padding = "8px 16px";
    btnRow.style.borderTop = "1px solid #333";

    const startBtn = document.createElement("button");
    startBtn.textContent = "Start Listening";
    startBtn.style.flex = "1";
    startBtn.style.padding = "8px";
    startBtn.style.borderRadius = "4px";
    startBtn.style.border = "1px solid #444";
    startBtn.style.background = "#2a2a3e";
    startBtn.style.color = "#e0e0e0";
    startBtn.style.cursor = "pointer";

    const stopBtn = document.createElement("button");
    stopBtn.textContent = "Stop";
    stopBtn.style.flex = "1";
    stopBtn.style.padding = "8px";
    stopBtn.style.borderRadius = "4px";
    stopBtn.style.border = "1px solid #444";
    stopBtn.style.background = "#2a2a3e";
    stopBtn.style.color = "#e0e0e0";
    stopBtn.style.cursor = "pointer";

    btnRow.appendChild(startBtn);
    btnRow.appendChild(stopBtn);
    container.appendChild(statusEl);
    container.appendChild(transcriptEl);
    container.appendChild(btnRow);
    el.appendChild(container);

    const capture = new AudioCapture({ sampleRate: 16000 });
    const playback = new AudioPlayback();

    const state: VoiceState = {
      capture,
      playback,
      status: "idle",
      statusEl,
      transcriptEl,
      dx,
    };
    states.set(elementId, state);

    capture.onChunk = (b64pcm: string) => {
      dx.send({ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq });
    };

    capture.onSilence = () => {
      if (state.status === "listening") {
        capture.stop();
        setStatus(state, "processing");
        dx.send({ type: "silence_detected" });
      }
    };

    playback.onComplete = () => {
      setStatus(state, "idle");
      dx.send({ type: "playback_complete" });
    };

    startBtn.addEventListener("click", async () => {
      if (state.status !== "idle") return;
      await capture.start();
      setStatus(state, "listening");
      dx.send({ type: "start_standby" });
    });

    stopBtn.addEventListener("click", () => {
      capture.stop();
      playback.stop();
      setStatus(state, "idle");
      dx.send({ type: "cancel" });
    });
  },

  update(elementId: string, data: unknown) {
    const state = states.get(elementId);
    if (!state) return;

    const msg = data as { type: string; text?: string; data?: string; seq?: number; message?: string };

    switch (msg.type) {
      case "transcript":
        addEntry(state.transcriptEl, "you", "You: " + (msg.text ?? ""));
        break;
      case "agent_response":
        addEntry(state.transcriptEl, "agent", "Agent: " + (msg.text ?? ""));
        break;
      case "audio_response":
        setStatus(state, "speaking");
        if (msg.data) state.playback.enqueue(msg.data);
        break;
      case "error":
        addEntry(state.transcriptEl, "error", "Error: " + (msg.message ?? ""));
        break;
    }
  },

  resize(_elementId: string) {},

  dispose(elementId: string) {
    const state = states.get(elementId);
    if (state) {
      state.capture.dispose();
      state.playback.dispose();
      states.delete(elementId);
    }
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("voice", voiceWidget);
