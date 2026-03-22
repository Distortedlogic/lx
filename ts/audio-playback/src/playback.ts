export type PlaybackCompleteCallback = () => void;

export class AudioPlayback {
  private audioCtx: AudioContext | null = null;
  private queue: ArrayBuffer[] = [];
  private playing = false;

  onComplete: PlaybackCompleteCallback | null = null;

  get isPlaying(): boolean {
    return this.playing;
  }

  get queueLength(): number {
    return this.queue.length;
  }

  private ensureContext(): AudioContext {
    if (!this.audioCtx) {
      this.audioCtx = new AudioContext();
    }
    return this.audioCtx;
  }

  enqueue(base64Wav: string): void {
    const binary = atob(base64Wav);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    this.queue.push(bytes.buffer);
    if (!this.playing) this.playNext();
  }

  private playNext(): void {
    if (this.queue.length === 0) {
      this.playing = false;
      this.onComplete?.();
      return;
    }
    this.playing = true;
    const ctx = this.ensureContext();
    const buffer = this.queue.shift()!;
    ctx.decodeAudioData(
      buffer.slice(0),
      (decoded) => {
        const source = ctx.createBufferSource();
        source.buffer = decoded;
        source.connect(ctx.destination);
        source.onended = () => this.playNext();
        source.start();
      },
      () => this.playNext()
    );
  }

  playAlertTone(frequency = 440, duration = 0.2, volume = 0.3): void {
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

  stop(): void {
    this.queue.length = 0;
    this.playing = false;
  }

  dispose(): void {
    this.stop();
    if (this.audioCtx) {
      this.audioCtx.close();
      this.audioCtx = null;
    }
  }
}
