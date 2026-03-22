var VoiceClient = (function(exports) {
	Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
	//#region ../audio-capture/dist/audio-capture.js
	var CAPTURE_WORKLET_CODE = `
class Capture extends AudioWorkletProcessor {
  constructor() {
    super();
    this.buffer = [];
  }
  process(inputs) {
    const input = inputs[0][0];
    if (input) {
      for (let i = 0; i < input.length; i++) {
        this.buffer.push(Math.max(-1, Math.min(1, input[i])));
      }
      if (this.buffer.length >= 4000) {
        const samples = this.buffer.splice(0, this.buffer.length);
        this.port.postMessage(samples);
      }
    }
    return true;
  }
}
registerProcessor('capture', Capture);
`;
	var DEFAULT_VAD_CONFIG = {
		threshold: .01,
		silenceTimeoutMs: 2e3
	};
	var VoiceActivityDetector = class {
		silenceStart = Date.now();
		config;
		constructor(config = {}) {
			this.config = {
				...DEFAULT_VAD_CONFIG,
				...config
			};
		}
		feed(samples) {
			let rms = 0;
			for (const s of samples) rms += s * s;
			rms = Math.sqrt(rms / samples.length);
			const isSpeech = rms > this.config.threshold;
			if (isSpeech) this.silenceStart = Date.now();
			return {
				isSpeech,
				silenceExceeded: !isSpeech && Date.now() - this.silenceStart > this.config.silenceTimeoutMs
			};
		}
		reset() {
			this.silenceStart = Date.now();
		}
	};
	var AudioCapture = class {
		audioCtx = null;
		mediaStream = null;
		workletNode = null;
		vad;
		sampleRate;
		seq = 0;
		running = false;
		onChunk = null;
		onSilence = null;
		constructor(config = {}) {
			this.sampleRate = config.sampleRate ?? 16e3;
			this.vad = new VoiceActivityDetector(config.vad);
		}
		get isRunning() {
			return this.running;
		}
		get currentSeq() {
			return this.seq;
		}
		async start() {
			if (this.running) return;
			this.running = true;
			this.seq = 0;
			this.vad.reset();
			this.audioCtx = new AudioContext({ sampleRate: this.sampleRate });
			this.mediaStream = await navigator.mediaDevices.getUserMedia({ audio: {
				sampleRate: this.sampleRate,
				channelCount: 1
			} });
			const workletUrl = "data:text/javascript," + encodeURIComponent(CAPTURE_WORKLET_CODE);
			await this.audioCtx.audioWorklet.addModule(workletUrl);
			const source = this.audioCtx.createMediaStreamSource(this.mediaStream);
			this.workletNode = new AudioWorkletNode(this.audioCtx, "capture");
			this.workletNode.port.onmessage = (evt) => {
				if (!this.running) return;
				const samples = evt.data;
				const { silenceExceeded } = this.vad.feed(samples);
				if (silenceExceeded) {
					this.onSilence?.();
					return;
				}
				const pcm = new Int16Array(samples.length);
				for (let i = 0; i < samples.length; i++) pcm[i] = Math.round(samples[i] * 32767);
				const b64 = btoa(String.fromCharCode(...new Uint8Array(pcm.buffer)));
				this.onChunk?.(b64);
				this.seq++;
			};
			source.connect(this.workletNode);
			this.workletNode.connect(this.audioCtx.destination);
		}
		stop() {
			this.running = false;
			if (this.workletNode) {
				this.workletNode.disconnect();
				this.workletNode = null;
			}
			if (this.mediaStream) {
				this.mediaStream.getTracks().forEach((t) => t.stop());
				this.mediaStream = null;
			}
		}
		dispose() {
			this.stop();
			if (this.audioCtx) {
				this.audioCtx.close();
				this.audioCtx = null;
			}
		}
	};
	//#endregion
	//#region ../audio-playback/dist/audio-playback.js
	var AudioPlayback = class {
		audioCtx = null;
		queue = [];
		playing = false;
		onComplete = null;
		get isPlaying() {
			return this.playing;
		}
		get queueLength() {
			return this.queue.length;
		}
		ensureContext() {
			if (!this.audioCtx) this.audioCtx = new AudioContext();
			return this.audioCtx;
		}
		enqueue(base64Wav) {
			const binary = atob(base64Wav);
			const bytes = new Uint8Array(binary.length);
			for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
			this.queue.push(bytes.buffer);
			if (!this.playing) this.playNext();
		}
		playNext() {
			if (this.queue.length === 0) {
				this.playing = false;
				this.onComplete?.();
				return;
			}
			this.playing = true;
			const ctx = this.ensureContext();
			const buffer = this.queue.shift();
			ctx.decodeAudioData(buffer.slice(0), (decoded) => {
				const source = ctx.createBufferSource();
				source.buffer = decoded;
				source.connect(ctx.destination);
				source.onended = () => this.playNext();
				source.start();
			}, () => this.playNext());
		}
		playAlertTone(frequency = 440, duration = .2, volume = .3) {
			const ctx = this.ensureContext();
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			osc.frequency.value = frequency;
			gain.gain.value = volume;
			osc.connect(gain);
			gain.connect(ctx.destination);
			osc.start();
			osc.stop(ctx.currentTime + duration);
		}
		stop() {
			this.queue.length = 0;
			this.playing = false;
		}
		dispose() {
			this.stop();
			if (this.audioCtx) {
				this.audioCtx.close();
				this.audioCtx = null;
			}
		}
	};
	//#endregion
	//#region ../widget-bridge/src/registry.ts
	var widgets = /* @__PURE__ */ new Map();
	function registerWidget(name, widget) {
		widgets.set(name, widget);
	}
	//#endregion
	//#region src/widget.ts
	var states = /* @__PURE__ */ new Map();
	function setStatus(state, status) {
		state.status = status;
		const labels = {
			idle: "Idle",
			listening: "Listening...",
			processing: "Processing...",
			speaking: "Speaking..."
		};
		state.statusEl.textContent = labels[status];
		state.dx.send({
			type: "status_change",
			status
		});
	}
	function addEntry(transcriptEl, cls, text) {
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
	registerWidget("voice", {
		mount(elementId, _config, dx) {
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
			const capture = new AudioCapture({ sampleRate: 16e3 });
			const playback = new AudioPlayback();
			const state = {
				capture,
				playback,
				status: "idle",
				statusEl,
				transcriptEl,
				dx
			};
			states.set(elementId, state);
			capture.onChunk = (b64pcm) => {
				dx.send({
					type: "audio_chunk",
					data: b64pcm,
					seq: capture.currentSeq
				});
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
		update(elementId, data) {
			const state = states.get(elementId);
			if (!state) return;
			const msg = data;
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
		resize(_elementId) {},
		dispose(elementId) {
			const state = states.get(elementId);
			if (state) {
				state.capture.dispose();
				state.playback.dispose();
				states.delete(elementId);
			}
			const el = document.getElementById(elementId);
			if (el) el.innerHTML = "";
		}
	});
	//#endregion
	exports.AudioCapture = AudioCapture;
	exports.AudioPlayback = AudioPlayback;
	exports.VoiceActivityDetector = VoiceActivityDetector;
	return exports;
})({});

//# sourceMappingURL=voice-client.js.map