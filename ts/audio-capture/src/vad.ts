export interface VadConfig {
  threshold: number;
  silenceTimeoutMs: number;
}

export const DEFAULT_VAD_CONFIG: VadConfig = {
  threshold: 0.01,
  silenceTimeoutMs: 2000,
};

export class VoiceActivityDetector {
  private silenceStart: number = Date.now();
  private config: VadConfig;

  constructor(config: Partial<VadConfig> = {}) {
    this.config = { ...DEFAULT_VAD_CONFIG, ...config };
  }

  feed(samples: number[]): { isSpeech: boolean; silenceExceeded: boolean } {
    let rms = 0;
    for (const s of samples) rms += s * s;
    rms = Math.sqrt(rms / samples.length);

    const isSpeech = rms > this.config.threshold;

    if (isSpeech) {
      this.silenceStart = Date.now();
    }

    const silenceExceeded =
      !isSpeech && Date.now() - this.silenceStart > this.config.silenceTimeoutMs;

    return { isSpeech, silenceExceeded };
  }

  reset(): void {
    this.silenceStart = Date.now();
  }
}
