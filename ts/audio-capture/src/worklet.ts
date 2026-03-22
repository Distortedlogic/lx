export const CAPTURE_WORKLET_CODE = `
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
