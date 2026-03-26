# Kokoro TTS Server — Audio Output Characteristics

Research date: 2026-03-26

## Server Location

- Local server at `localhost:8094` (Docker container maps 8094 -> 8000 internal)
- Endpoint: `POST /infer`
- Source: `/home/entropybender/repos/mcp-toolbelt/models/kokoro/kokoro_server.py`

---

## 1. Sample Rate: 24000 Hz (24 kHz)

Hardcoded at three levels:

- **Server constant**: `KOKORO_SAMPLE_RATE = 24000` in `kokoro_server.py` line 20
- **Model internals**: `sampling_rate=24000` in `istftnet.py` `Generator.__init__` (the iSTFT neural vocoder)
- **Timestamp math**: `pipeline.py` line 289 uses `MAGIC_DIVISOR = 80`, derived from "multiply by 600 to go from pred_dur frames to sample_rate 24000"

This is not configurable. The 24 kHz rate is baked into the neural network architecture.

## 2. Bit Depth: 16-bit signed integer PCM

The server normalizes the raw float32 model output to int16 before writing:

```python
INT16_MAX = 32767
audio_int16 = numpy.int16(audio / numpy.max(numpy.abs(audio)) * INT16_MAX)
scipy.io.wavfile.write(buffer, KOKORO_SAMPLE_RATE, audio_int16)
```

- The model (`KModel.forward`) outputs `torch.FloatTensor` (32-bit float, roughly -1.0 to +1.0 range)
- The server peak-normalizes this to fill the full int16 range (-32768 to +32767)
- `scipy.io.wavfile.write` with int16 input produces WAV format code 1 (PCM), 16 bits per sample

## 3. Channels: Mono (1 channel)

- `KModel.forward` returns `audio.squeeze().cpu()` which produces a 1-D tensor
- `numpy.concatenate(segments)` on 1-D arrays stays 1-D
- `scipy.io.wavfile.write` with a 1-D array writes a mono WAV (1 channel)

## 4. Zero Crossing / Initial Sample Behavior: NO zero crossing guarantee

The audio does NOT start at a zero crossing. The neural network output starts wherever the iSTFT reconstruction begins. Specific evidence:

- The `SineGen._f02sine` method adds a random initial phase offset: `rand_ini = torch.rand(...)` and `rad_values[:, 0, :] = rad_values[:, 0, :] + rand_ini` (only the fundamental is zeroed: `rand_ini[:, 0] = 0`)
- The iSTFT reconstruction (`torch.istft`) produces a waveform whose starting sample depends on the spectral content, not on any zero-crossing constraint
- The server's peak normalization (`audio / max(abs(audio)) * INT16_MAX`) scales but does not shift the waveform

In practice, the first sample is often a non-zero value. This causes an audible pop/click when playback begins, because the DAC jumps from silence (0) to a non-zero sample instantaneously.

**Your codebase already works around this** — `voice_banner.rs` line 173 calls `prepend_silence(&wav, 50)` which inserts 50ms of zero-valued samples (1200 samples at 24 kHz) before the audio data, giving the DAC a zero-to-zero transition. The commit `106f3c2` ("Revert to HTMLAudioElement, fix clipping with 50ms WAV silence prepend") documents this fix.

## 5. Fade-in Configuration: None available

There is no fade-in option in any layer of the stack:

- **Kokoro model** (`KModel`, `KPipeline`): No fade-in parameter. The `__call__` accepts only `text`, `voice`, `speed`, `split_pattern`, `model`.
- **Server** (`kokoro_server.py`): `SpeechRequest` accepts only `text`, `voice`, `lang_code`, `speed`. No audio post-processing options.
- **Kokoro library** (v0.9.4): No built-in fade-in, normalization, or envelope shaping.

If fade-in is needed, it must be applied client-side. Options:
- Prepend silence (current approach in `voice_banner.rs`)
- Apply a linear/cosine ramp to the first N samples of the PCM data after decoding the WAV
- Use Web Audio API's `GainNode` with a timed ramp at playback time

## 6. Response Format: Complete WAV with proper headers, base64-encoded in JSON

The server does NOT return raw PCM. The full chain:

1. `scipy.io.wavfile.write(buffer, 24000, audio_int16)` writes a complete WAV file (RIFF header + fmt chunk + data chunk) into a `BytesIO` buffer
2. The buffer is base64-encoded: `base64.b64encode(buffer.getvalue()).decode()`
3. Returned as JSON: `GenerationResponse(data=<base64>, format="wav", metadata={"sample_rate": 24000})`

The Rust client (`BinaryInferenceClient`) parses the JSON, base64-decodes the `data` field, and returns `Vec<u8>` — a complete WAV file with proper headers.

WAV header structure (standard 44-byte header from scipy):
- Bytes 0-3: "RIFF"
- Bytes 8-11: "WAVE"
- Bytes 12-15: "fmt "
- Format: PCM (code 1)
- Channels: 1 (mono)
- Sample rate: 24000
- Bits per sample: 16
- Byte rate: 48000 (24000 * 1 * 2)
- Block align: 2 (1 channel * 2 bytes)

---

## Summary Table

| Property | Value |
|---|---|
| Sample rate | 24000 Hz |
| Bit depth | 16-bit signed integer (PCM) |
| Channels | 1 (mono) |
| WAV headers | Yes, complete RIFF/WAVE headers |
| Transport | Base64 in JSON (`GenerationResponse.data`) |
| Zero-crossing start | No — first sample is arbitrary |
| Fade-in option | None — must be applied client-side |
| Peak normalization | Yes — server normalizes to full int16 range |
| Model | hexgrad/Kokoro-82M (v0.9.4) |
| Vocoder | iSTFTNet (inverse Short-Time Fourier Transform) |

## Key Source Files

- Server: `/home/entropybender/repos/mcp-toolbelt/models/kokoro/kokoro_server.py`
- Rust client (mcp-toolbelt): `/home/entropybender/repos/mcp-toolbelt/models/kokoro/client/src/lib.rs`
- Rust client (dioxus-common): `/home/entropybender/repos/dioxus-common/crates/common-kokoro/src/lib.rs`
- Binary inference client: `/home/entropybender/repos/dioxus-common/crates/common-inference/src/lib.rs`
- Silence prepend workaround: `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/voice_banner.rs`
- Kokoro pipeline: `/home/entropybender/repos/mcp-toolbelt/.venv/lib/python3.14/site-packages/kokoro/pipeline.py`
- iSTFTNet vocoder: `/home/entropybender/repos/mcp-toolbelt/.venv/lib/python3.14/site-packages/kokoro/istftnet.py`
- Docker compose: `/home/entropybender/repos/mcp-toolbelt/models/kokoro/docker-compose.yml`

## Sources

- [hexgrad/Kokoro-82M on Hugging Face](https://huggingface.co/hexgrad/Kokoro-82M)
- [Kokoro GitHub repository](https://github.com/hexgrad/kokoro)
- [scipy.io.wavfile.write documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.io.wavfile.write.html)
