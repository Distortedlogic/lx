import { AudioCapture } from "@lx/audio-capture";
import { AudioPlayback } from "@lx/audio-playback";
import { registerWidget } from "../src/registry";
import type { Widget } from "../src/registry";
import type { Dioxus } from "../src/types";

type VoiceStatus = "idle" | "listening" | "processing" | "speaking";

interface VoiceState {
  capture: AudioCapture;
  playback: AudioPlayback;
  status: VoiceStatus;
  dx: Dioxus;
}

const states = new Map<string, VoiceState>();

function transition(state: VoiceState, status: VoiceStatus): void {
  state.status = status;
  state.dx.send({ type: "status_change", status });
}

const voiceWidget: Widget = {
  mount(elementId: string, _config: unknown, dx: Dioxus) {
    const capture = new AudioCapture({ sampleRate: 16000 });
    const playback = new AudioPlayback();

    const state: VoiceState = { capture, playback, status: "idle", dx };
    states.set(elementId, state);

    capture.onChunk = (b64pcm: string) => {
      dx.send({ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq });
    };

    capture.onSilence = () => {
      if (state.status === "listening") {
        capture.stop();
        transition(state, "processing");
        dx.send({ type: "silence_detected" });
      }
    };

    playback.onComplete = () => {
      transition(state, "idle");
      dx.send({ type: "playback_complete" });
    };
  },

  update(elementId: string, data: unknown) {
    const state = states.get(elementId);
    if (!state) return;

    const msg = data as { type: string; data?: string };

    switch (msg.type) {
      case "start_capture":
        if (state.status !== "idle") return;
        state.capture.start().then(() => {
          transition(state, "listening");
          state.dx.send({ type: "start_standby" });
        });
        break;
      case "stop_capture":
        state.capture.stop();
        state.playback.stop();
        transition(state, "idle");
        state.dx.send({ type: "cancel" });
        break;
      case "audio_response":
        transition(state, "speaking");
        if (msg.data) state.playback.enqueue(msg.data);
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
  },
};

registerWidget("voice", voiceWidget);
