import { CAPTURE_WORKLET_CODE } from "./worklet";
import { VoiceActivityDetector, type VadConfig } from "./vad";

export interface CaptureConfig {
  sampleRate?: number;
  vad?: Partial<VadConfig>;
}

export type ChunkCallback = (b64pcm: string) => void;
export type SilenceCallback = () => void;

export class AudioCapture {
  private audioCtx: AudioContext | null = null;
  private mediaStream: MediaStream | null = null;
  private workletNode: AudioWorkletNode | null = null;
  private vad: VoiceActivityDetector;
  private sampleRate: number;
  private seq = 0;
  private running = false;

  onChunk: ChunkCallback | null = null;
  onSilence: SilenceCallback | null = null;

  constructor(config: CaptureConfig = {}) {
    this.sampleRate = config.sampleRate ?? 16000;
    this.vad = new VoiceActivityDetector(config.vad);
  }

  get isRunning(): boolean {
    return this.running;
  }

  get currentSeq(): number {
    return this.seq;
  }

  async start(): Promise<void> {
    if (this.running) return;
    this.running = true;
    this.seq = 0;
    this.vad.reset();

    this.audioCtx = new AudioContext({ sampleRate: this.sampleRate });
    this.mediaStream = await navigator.mediaDevices.getUserMedia({
      audio: { sampleRate: this.sampleRate, channelCount: 1 },
    });

    const workletUrl =
      "data:text/javascript," + encodeURIComponent(CAPTURE_WORKLET_CODE);
    await this.audioCtx.audioWorklet.addModule(workletUrl);

    const source = this.audioCtx.createMediaStreamSource(this.mediaStream);
    this.workletNode = new AudioWorkletNode(this.audioCtx, "capture");

    this.workletNode.port.onmessage = (evt: MessageEvent) => {
      if (!this.running) return;
      const samples: number[] = evt.data;

      const { silenceExceeded } = this.vad.feed(samples);
      if (silenceExceeded) {
        this.onSilence?.();
        return;
      }

      const pcm = new Int16Array(samples.length);
      for (let i = 0; i < samples.length; i++) {
        pcm[i] = Math.round(samples[i] * 32767);
      }
      const b64 = btoa(
        String.fromCharCode(...new Uint8Array(pcm.buffer))
      );
      this.onChunk?.(b64);
      this.seq++;
    };

    source.connect(this.workletNode);
    this.workletNode.connect(this.audioCtx.destination);
  }

  stop(): void {
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

  dispose(): void {
    this.stop();
    if (this.audioCtx) {
      this.audioCtx.close();
      this.audioCtx = null;
    }
  }
}
