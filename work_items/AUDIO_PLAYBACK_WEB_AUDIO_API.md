# Goal

Rewrite `AudioPlayback` to use the Web Audio API (`AudioContext` + `decodeAudioData` + `AudioBufferSourceNode`) instead of `HTMLAudioElement`. Same public API, no clipping.

# Why

`HTMLAudioElement` (`new Audio(url)`) has inherent startup latency. The browser's audio output pipeline initializes lazily on first `play()`, clipping the first few milliseconds of audio. No amount of `canplaythrough` waiting or silence prepending fixes this — it's fundamental to the HTMLAudioElement lifecycle. The Web Audio API pre-decodes audio into PCM buffers before scheduling playback. When `start()` fires, decoded samples are immediately available. No initialization lag, no clipping.

# File

`/home/entropybender/repos/dioxus-common/ts/audio-playback/src/playback.ts`

This is the only file that changes. The public API stays identical:
- `enqueue(base64Wav: string, id?: string): void`
- `stop(): void`
- `dispose(): void`
- `onComplete: PlaybackCompleteCallback | null`
- `onItemStart: ItemStartCallback | null`
- `get isPlaying(): boolean`
- `get queueLength(): number`
- `playAlertTone(frequency?, duration?, volume?): void`

No callers change. The voice widget (`voice.ts`) calls `playback.enqueue(base64data)` and `playback.onComplete` — both unchanged.

# Task List

### Task 1: Rewrite AudioPlayback with Web Audio API

**Subject:** Replace HTMLAudioElement internals with AudioContext + AudioBufferSourceNode

**Description:** Replace the entire contents of `/home/entropybender/repos/dioxus-common/ts/audio-playback/src/playback.ts` with:

```typescript
export type PlaybackCompleteCallback = () => void;
export type ItemStartCallback = (id: string) => void;

interface QueueItem {
  buffer: Promise<AudioBuffer>;
  id: string;
}

export class AudioPlayback {
  private context: AudioContext;
  private queue: QueueItem[] = [];
  private playing = false;
  private currentSource: AudioBufferSourceNode | null = null;

  onComplete: PlaybackCompleteCallback | null = null;
  onItemStart: ItemStartCallback | null = null;

  constructor() {
    this.context = new AudioContext();
  }

  get isPlaying(): boolean {
    return this.playing;
  }

  get queueLength(): number {
    return this.queue.length;
  }

  enqueue(base64Wav: string, id = ""): void {
    const binary = atob(base64Wav);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const buffer = this.context.decodeAudioData(bytes.buffer.slice(0));
    this.queue.push({ buffer, id });
    if (!this.playing) this.playNext();
  }

  private async playNext(): Promise<void> {
    if (this.queue.length === 0) {
      this.playing = false;
      this.onComplete?.();
      return;
    }
    this.playing = true;
    if (this.context.state === "suspended") {
      await this.context.resume();
    }
    const item = this.queue.shift()!;
    try {
      const audioBuffer = await item.buffer;
      const source = this.context.createBufferSource();
      source.buffer = audioBuffer;
      source.connect(this.context.destination);
      this.currentSource = source;
      this.onItemStart?.(item.id);
      source.onended = () => {
        this.currentSource = null;
        this.playNext();
      };
      source.start();
    } catch {
      this.currentSource = null;
      this.playNext();
    }
  }

  playAlertTone(frequency = 440, duration = 0.2, volume = 0.3): void {
    const osc = this.context.createOscillator();
    const gain = this.context.createGain();
    osc.frequency.value = frequency;
    gain.gain.value = volume;
    osc.connect(gain);
    gain.connect(this.context.destination);
    osc.start();
    osc.stop(this.context.currentTime + duration);
  }

  stop(): void {
    this.queue.length = 0;
    this.playing = false;
    if (this.currentSource) {
      this.currentSource.stop();
      this.currentSource.disconnect();
      this.currentSource = null;
    }
  }

  dispose(): void {
    this.stop();
    this.context.close();
  }
}
```

Changes from the old implementation:

- **`HTMLAudioElement` → `AudioBufferSourceNode`**: No more `new Audio(url)`, blob URLs, or `canplaythrough`. Audio data is decoded via `decodeAudioData()` into an `AudioBuffer`, then played via `AudioBufferSourceNode.start()`.
- **`AudioContext` created once in constructor**: Reused for all playback and alert tones. The old `playAlertTone` created a new `AudioContext` per call.
- **`QueueItem` stores `Promise<AudioBuffer>` instead of raw base64**: `decodeAudioData` is called eagerly in `enqueue()`, so decoding happens while the current item is still playing. By the time `playNext()` awaits the promise, the buffer is already decoded.
- **`playNext()` is now `async`**: It awaits the `AudioBuffer` promise and `context.resume()`. The `onended` callback calls `playNext()` for the next item (same chaining pattern as before).
- **`context.resume()` in `playNext()`**: Handles the suspended AudioContext case. By the time `playNext()` is called, a user gesture has already occurred (the user clicked PUSH TO TALK), so `resume()` succeeds.
- **`stop()` calls `source.stop()` and `source.disconnect()`**: Replaces `audio.pause()`. `disconnect()` releases the node from the audio graph.
- **`dispose()` calls `context.close()`**: Releases the AudioContext's system resources. The old version didn't close anything.
- **`bytes.buffer.slice(0)` in `enqueue()`**: `decodeAudioData` takes ownership of the ArrayBuffer and detaches it. `.slice(0)` creates a copy so the original Uint8Array isn't invalidated. Without this, the `decodeAudioData` call would detach `bytes.buffer` and any subsequent access to `bytes` would fail.

**ActiveForm:** Rewriting AudioPlayback with Web Audio API

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/AUDIO_PLAYBACK_WEB_AUDIO_API.md" })
```
