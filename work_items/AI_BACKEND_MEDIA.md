# Goal

Add three new backend traits to RuntimeCtx for media capabilities: `TranscribeBackend` (audio→text), `SpeechBackend` (text→audio), and `ImageGenBackend` (prompt→image). Each is a separate trait because the request/response shapes are fundamentally different and different deployments swap different providers.

Default backends call the mcp-toolbelt inference servers directly — these ARE our infrastructure, not a fallback. Whisper at port 8095, Kokoro at port 8094, Flux2 at port 8091. All follow the shared inference protocol: `GET /health`, `POST /infer`.

Extend `std/ai` with `ai.transcribe`, `ai.speak`, `ai.imagine`.

# Why

- lx programs have no way to work with audio or images. Adding these unlocks voice-driven workflows, audio processing, and image generation.
- The inference servers are already running in mcp-toolbelt. The shared protocol (`/health` + `/infer`) makes each backend a thin HTTP client.
- Three separate traits (not one `MediaBackend`) because:
  - Request shapes are different: audio bytes vs text+voice vs prompt+dimensions
  - Response shapes are different: `{text, language}` vs `GenerationResponse` (base64+format+metadata)
  - An agent doing transcription doesn't need image generation wired in
  - Providers differ: Whisper vs Google STT, Kokoro vs ElevenLabs, Flux vs DALL-E

# What Changes

**`crates/lx/src/backends/mod.rs` — three new traits:**

```rust
pub struct TranscribeOpts {
    pub language: Option<String>,
}

pub trait TranscribeBackend: Send + Sync {
    fn transcribe(&self, audio_base64: &str, opts: &TranscribeOpts, span: Span) -> Result<Value, LxError>;
}

pub struct SpeechOpts {
    pub voice: Option<String>,
    pub speed: Option<f64>,
}

pub trait SpeechBackend: Send + Sync {
    fn speak(&self, text: &str, opts: &SpeechOpts, span: Span) -> Result<Value, LxError>;
}

pub struct ImageGenOpts {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub steps: Option<u32>,
    pub guidance: Option<f64>,
    pub seed: Option<u64>,
}

pub trait ImageGenBackend: Send + Sync {
    fn imagine(&self, prompt: &str, opts: &ImageGenOpts, span: Span) -> Result<Value, LxError>;
}
```

Add three fields to `RuntimeCtx`:
- `pub transcribe: Arc<dyn TranscribeBackend>` — default `WhisperBackend`
- `pub speech: Arc<dyn SpeechBackend>` — default `KokoroBackend`
- `pub image_gen: Arc<dyn ImageGenBackend>` — default `FluxBackend`

**`crates/lx/src/backends/transcribe.rs` — WhisperBackend:**

POST to `http://localhost:8095/infer` with `{"audio_data": base64, "language": lang}`. Returns `Ok({text: Str, language: Str})`.

**`crates/lx/src/backends/speech.rs` — KokoroBackend:**

POST to `http://localhost:8094/infer` with `{"text": text, "voice": voice, "lang_code": "a", "speed": speed}`. Returns `Ok({data: base64_wav, format: "wav", metadata: {sample_rate}})`.

**`crates/lx/src/backends/image_gen.rs` — FluxBackend:**

POST to `http://localhost:8091/infer` with `{"prompt": prompt, "width": w, "height": h, "num_inference_steps": steps, "guidance_scale": guidance, "seed": seed}`. Returns `Ok({data: base64_png, format: "png", metadata: {width, height}})`.

**`crates/lx/src/stdlib/ai.rs` — extend with three functions:**

`ai.transcribe`, `ai.speak`, `ai.imagine` (+ `_with` variants for opts).

# Files Affected

- `crates/lx/src/backends/mod.rs` — Add 3 traits, 3 opts structs, 3 fields to RuntimeCtx
- `crates/lx/src/backends/transcribe.rs` — New file: WhisperBackend
- `crates/lx/src/backends/speech.rs` — New file: KokoroBackend
- `crates/lx/src/backends/image_gen.rs` — New file: FluxBackend
- `crates/lx/src/stdlib/ai.rs` — Add transcribe/speak/imagine entries to build()
- `tests/110_media.lx` — New test file

# Task List

### Task 1: Add three backend traits to RuntimeCtx

**Subject:** Add TranscribeBackend, SpeechBackend, ImageGenBackend traits and fields

**Description:** Edit `crates/lx/src/backends/mod.rs`:

Add the three opts structs and traits (shown above in What Changes). All derive `Debug, Clone, Default` for opts structs.

Add `mod transcribe; mod speech; mod image_gen;` and corresponding `pub use` lines.

Add three fields to `RuntimeCtx`:
```rust
pub transcribe: Arc<dyn TranscribeBackend>,
pub speech: Arc<dyn SpeechBackend>,
pub image_gen: Arc<dyn ImageGenBackend>,
```

In `Default` impl:
```rust
transcribe: Arc::new(WhisperBackend::new("http://localhost:8095".into())),
speech: Arc::new(KokoroBackend::new("http://localhost:8094".into())),
image_gen: Arc::new(FluxBackend::new("http://localhost:8091".into())),
```

**ActiveForm:** Adding media backend traits to RuntimeCtx

---

### Task 2: Implement WhisperBackend

**Subject:** Create transcribe.rs with WhisperBackend calling Whisper inference server

**Description:** Create `crates/lx/src/backends/transcribe.rs`.

```rust
pub struct WhisperBackend {
    url: String,
}

impl WhisperBackend {
    pub fn new(url: String) -> Self { Self { url } }
}
```

Implement `TranscribeBackend for WhisperBackend`:

`fn transcribe(&self, audio_base64: &str, opts: &TranscribeOpts, span: Span) -> Result<Value, LxError>`:

Use `tokio::task::block_in_place` + `Handle::current().block_on`:
- POST to `{self.url}/infer`
- Body: `{"audio_data": audio_base64, "language": opts.language}`
- Parse response: `{"text": "...", "language": "en"}`
- Return `Ok(Value::Ok(Box::new(record! { "text" => ..., "language" => ... })))`
- On connection error, return `Ok(Value::Err(Box::new(Value::Str(Arc::from(format!("whisper: {e}"))))))`

**ActiveForm:** Implementing WhisperBackend

---

### Task 3: Implement KokoroBackend

**Subject:** Create speech.rs with KokoroBackend calling Kokoro TTS inference server

**Description:** Create `crates/lx/src/backends/speech.rs`.

```rust
pub struct KokoroBackend {
    url: String,
}

impl KokoroBackend {
    pub fn new(url: String) -> Self { Self { url } }
}
```

Implement `SpeechBackend for KokoroBackend`:

`fn speak(&self, text: &str, opts: &SpeechOpts, span: Span) -> Result<Value, LxError>`:

Use `block_in_place` + `block_on`:
- POST to `{self.url}/infer`
- Body: `{"text": text, "voice": opts.voice.as_deref().unwrap_or("af_heart"), "lang_code": "a", "speed": opts.speed.unwrap_or(1.0)}`
- Parse response: `GenerationResponse` shape — `{"data": "base64_wav", "format": "wav", "metadata": {"sample_rate": 24000}}`
- Return `Ok(Value::Ok(Box::new(record! { "data" => base64_str, "format" => "wav", "metadata" => metadata_record })))`

**ActiveForm:** Implementing KokoroBackend

---

### Task 4: Implement FluxBackend

**Subject:** Create image_gen.rs with FluxBackend calling Flux2 inference server

**Description:** Create `crates/lx/src/backends/image_gen.rs`.

```rust
pub struct FluxBackend {
    url: String,
}

impl FluxBackend {
    pub fn new(url: String) -> Self { Self { url } }
}
```

Implement `ImageGenBackend for FluxBackend`:

`fn imagine(&self, prompt: &str, opts: &ImageGenOpts, span: Span) -> Result<Value, LxError>`:

Use `block_in_place` + `block_on`:
- POST to `{self.url}/infer`
- Body: `{"prompt": prompt, "width": opts.width.unwrap_or(1024), "height": opts.height.unwrap_or(1024), "num_inference_steps": opts.steps.unwrap_or(28), "guidance_scale": opts.guidance.unwrap_or(3.5)}`
- If `opts.seed` is Some, add `"seed"` field
- Parse response: `GenerationResponse` shape — `{"data": "base64_png", "format": "png", "metadata": {"width": 1024, "height": 1024}}`
- Return `Ok(Value::Ok(Box::new(record! { "data" => base64_str, "format" => "png", "metadata" => metadata_record })))`

**ActiveForm:** Implementing FluxBackend

---

### Task 5: Extend std/ai and write tests

**Subject:** Add ai.transcribe, ai.speak, ai.imagine to std/ai

**Description:** Edit `crates/lx/src/stdlib/ai.rs`:

In `build()`, add:
- `"transcribe"` → `bi_transcribe` arity 1 — args[0] is base64 audio string. Calls `ctx.transcribe.transcribe(audio, &TranscribeOpts::default(), span)`.
- `"transcribe_with"` → `bi_transcribe_with` arity 1 — args[0] is Record `{audio: Str, language?: Str}`.
- `"speak"` → `bi_speak` arity 1 — args[0] is text string. Calls `ctx.speech.speak(text, &SpeechOpts::default(), span)`.
- `"speak_with"` → `bi_speak_with` arity 1 — args[0] is Record `{text: Str, voice?: Str, speed?: Float}`.
- `"imagine"` → `bi_imagine` arity 1 — args[0] is prompt string. Calls `ctx.image_gen.imagine(prompt, &ImageGenOpts::default(), span)`.
- `"imagine_with"` → `bi_imagine_with` arity 1 — args[0] is Record `{prompt: Str, width?: Int, height?: Int, steps?: Int, guidance?: Float, seed?: Int}`.

Import the opts structs from `crate::backends`.

Create `tests/110_media.lx`:

```
use std/ai

-- These functions exist and are callable
-- They call the inference servers; if servers aren't running, they return Err

transcribe_result = ai.transcribe "dGVzdA=="
transcribe_result ? {
  Ok r -> { assert (r.text | type_of == "Str") "transcribe returns text" }
  Err _ -> { log.info "110_media: transcribe skipped (server not running)" }
}

speak_result = ai.speak "hello world"
speak_result ? {
  Ok r -> { assert (r.format == "wav") "speak returns wav" }
  Err _ -> { log.info "110_media: speak skipped (server not running)" }
}

imagine_result = ai.imagine "a red circle"
imagine_result ? {
  Ok r -> { assert (r.format == "png") "imagine returns png" }
  Err _ -> { log.info "110_media: imagine skipped (server not running)" }
}

log.info "110_media: all passed"
```

Run `just diagnose` to verify compilation.

**ActiveForm:** Extending std/ai with media functions

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AI_BACKEND_MEDIA.md" })
```

Then call `next_task` to begin.
