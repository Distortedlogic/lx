# WebKit2GTK Audio API Research

## 1. Version Mapping

### Rust Crate to Native Library

The project pins `webkit2gtk = "=2.0.1"` with `features = ["v2_38"]` in `crates/lx-desktop/Cargo.toml`. This Rust crate is a **binding layer**, not a WebKit engine. The crate version (2.0.1) is unrelated to the WebKit engine version.

The `v2_38` feature flag means the Rust bindings expose APIs available up through WebKitGTK 2.38. At build time, `webkit2gtk-sys` links against whatever `webkit2gtk-4.1` system library is installed via `pkg-config`. The minimum required native version is 2.4; the maximum API surface exposed with `v2_38` is the 2.38 API.

### Installed Engine Version (This System)

- **webkit2gtk4.1**: 2.50.5-1.fc43 (Fedora 43)
- **WebKitGTK engine**: 2.50.5

So the Rust bindings expose the 2.38 API surface, but the actual runtime engine is WebKitGTK 2.50.5, which is far more recent. All Web Audio fixes through 2.50.5 are active at runtime.

### GStreamer Version (This System)

- **GStreamer**: 1.26.10-1.fc43
- **gst-plugins-good** (contains `wavparse`): installed
- **gst-plugins-base** (core audio elements): installed
- All required GStreamer plugins for WAV/PCM audio decoding are present.

## 2. Web Audio API in WebKit2GTK

### Architecture

WebKitGTK uses GStreamer as its multimedia backend on Linux. The Web Audio API implementation is in:
- `Source/WebCore/platform/audio/gstreamer/AudioFileReaderGStreamer.cpp` -- handles `decodeAudioData()`
- `Source/WebCore/platform/audio/gstreamer/WebKitWebAudioSourceGStreamer.cpp` -- Web Audio source nodes
- `Source/WebCore/platform/audio/gstreamer/AudioDestinationGStreamer.cpp` -- audio output

For `decodeAudioData()`, WebKit creates a GStreamer pipeline: `giostreamsrc` (memory source) -> `decodebin2` (auto-detect codec) -> `deinterleave` (split channels) -> `appsink` (extract float PCM). The decoded data is assembled into an `AudioBus` for Web Audio processing.

### decodeAudioData() Status

**The Web Audio API 1.0 implementation report rates WebKit/Safari as "excellent"**: 1115/1126 IDL tests pass, 6478/6500 functional tests pass. `decodeAudioData` is not listed among failing tests.

However, this report covers Safari (macOS/iOS), which uses CoreAudio. The GTK/WPE ports use GStreamer, a completely separate audio backend. The conformance results do not directly apply to WebKitGTK.

### Known decodeAudioData Bugs (GStreamer Backend)

| Bug | Description | Status |
|-----|-------------|--------|
| [#105298](https://bugs.webkit.org/show_bug.cgi?id=105298) | `decode-audio-data-basic.html` test failed on GStreamer backend -- error callback not invoked for invalid audio | **RESOLVED FIXED** (2012, r138580) |
| [#106658](https://bugs.webkit.org/show_bug.cgi?id=106658) | `decodeAudioData` throws DOM Exception 12 when audio data contains ID3 tags/metadata before frame header | **UNCONFIRMED** (2013, marked "by design") |
| [#226922](https://bugs.webkit.org/show_bug.cgi?id=226922) | Safari 15 supported WebM Opus in `<audio>` but `decodeAudioData` rejected it -- spec violation | **RESOLVED FIXED** (Safari Technology Preview 132+) |

### Observed Behavior in This Project

**`decodeAudioData()` silently fails in WebKit2GTK** (documented in `VOICE_AUDIO_PLAYBACK_NOTES.md`). The Web Audio API rewrite attempted `AudioContext` + `decodeAudioData` + `AudioBufferSourceNode` -- no audio played. The catch block swallowed errors and called `playNext()`, so the pipeline completed but produced no sound.

### Likely Root Cause of Silent Failure

Several factors could explain this:

1. **WAV format compatibility**: The WAV data comes from Kokoro TTS (16-bit PCM). GStreamer's `wavparse` supports standard PCM WAV, but `decodebin2` auto-detection may fail on certain WAV variants, or the pipeline may not wire up correctly for raw PCM from a memory buffer.

2. **AudioContext autoplay policy**: Even with `set_media_playback_requires_user_gesture(false)`, the `AudioContext` may start in `suspended` state. If `context.resume()` is not called (or fails silently), `AudioBufferSourceNode.start()` produces no output. The work item code does call `context.resume()` but only in `playNext()`, not at construction time.

3. **GStreamer pipeline setup failure**: The GStreamer pipeline for `decodeAudioData` uses `giostreamsrc` to feed raw bytes. If the pipeline fails to negotiate caps or find a suitable decoder element, `decodeAudioData` rejects -- but the Promise rejection may not propagate correctly if the catch block silently continues.

4. **WebAudio enable setting**: The project does NOT explicitly call `set_enable_webaudio(true)` in its WebKit settings (only `set_enable_media_stream` and `set_media_playback_requires_user_gesture` are set in `webview_permissions.rs`). WebAudio is enabled by default, but it is worth verifying.

### Recent WebAudio Fixes in WebKitGTK

| Version | Fix |
|---------|-----|
| 2.50.0 | Fix WebAudio issues after idling for a minute |
| 2.50.1 | Fix audio playback broken on Instagram |
| 2.50.6 | Fix WebAudio not resuming correctly after `window.alert()` |
| 2.50.6 | Fix WebAudio producing incorrect output due to incorrect sample buffer management |

The 2.50.6 fixes are particularly relevant -- incorrect sample buffer management could explain silent output from `decodeAudioData`.

## 3. HTMLAudioElement Behavior

### General Architecture

When `new Audio(url)` is used with a blob URL, WebKitGTK creates a GStreamer `playbin` pipeline that:
1. Fetches data from the blob URL via WebKit's network layer
2. Auto-detects the format with `decodebin`
3. Routes decoded audio to `autoaudiosink` (which selects PipeWire/PulseAudio/ALSA)

### Startup Latency

There are **two distinct sources of first-play latency**:

#### Source 1: GStreamer Pipeline Cold Start

The GStreamer pipeline is constructed lazily on first `play()`. The pipeline must:
- Negotiate caps between elements
- Initialize the audio sink (connect to PipeWire/PulseAudio)
- Buffer enough data for playback

This introduces a one-time delay on the first audio element that plays. Subsequent `Audio` elements reuse cached pipeline knowledge, reducing latency.

#### Source 2: PipeWire Suspend-on-Idle (Fedora-Specific)

**This is the most likely cause of the audio clipping observed in this project.**

PipeWire (default on Fedora 35+) suspends audio sinks after 5 seconds of idle. When playback resumes:
- The audio device must be re-initialized
- The first few milliseconds of audio are lost during wake-up
- Users report "pops", "delays", and "clipped starts"

This explains the observed symptom: the first ~50ms of TTS audio is clipped, and the 50ms silence prepend works as a mitigation because it provides throwaway audio during the PipeWire wake-up window.

**Fix**: Disable PipeWire suspend-on-idle:

For modern WirePlumber (Fedora 43):
```
mkdir -p ~/.config/wireplumber/wireplumber.conf.d/
cat > ~/.config/wireplumber/wireplumber.conf.d/51-disable-suspend.conf << 'EOF'
monitor.alsa.rules = [
  {
    matches = [
      { node.name = "~alsa_output.*" }
    ]
    actions = {
      update-props = {
        session.suspend-timeout-seconds = 0
      }
    }
  }
]
EOF
systemctl --user restart wireplumber
```

### oncanplaythrough Reliability

- `oncanplaythrough` fires when the browser estimates it can play through without buffering stops
- On iOS 17.4+, there are known issues with `canplay`/`canplaythrough` not firing (Apple Developer Forums thread)
- On WebKitGTK with blob URLs, `oncanplaythrough` appears to fire reliably based on project experience (the current implementation waits for it successfully)
- However, `oncanplaythrough` does NOT guarantee zero latency -- it only means data is buffered, not that the audio sink is initialized

### Known Blob URL Bug

[WebKit Bug #238113](https://bugs.webkit.org/show_bug.cgi?id=238113): Audio element playback ends early when `src` is a blob URL. Blobs > 65,536 bytes trigger infinite HTTP 206 Partial Content loops due to missing Content-Range headers. Marked as duplicate of #238170 (RESOLVED FIXED). This was an iOS 15.4-specific issue and does not affect WebKitGTK.

## 4. Blob URL Audio Behavior in WebKit2GTK

### How It Works

1. JavaScript creates a `Blob` from decoded base64 WAV data
2. `URL.createObjectURL(blob)` creates a `blob:` URL
3. `new Audio(blobUrl)` creates an HTMLAudioElement with that source
4. WebKit's blob URL handler serves the data via internal range requests

### Known Issues

- **Tauri/WebKitGTK local file audio**: WebKitGTK cannot load audio/video from custom URI schemes (Tauri asset:// protocol). Only `http://`, `https://`, and `blob:` URLs work for media. This does not affect this project since blob URLs are used. ([Tauri #8654](https://github.com/tauri-apps/tauri/issues/8654))

- **Memory**: Each blob URL holds a strong reference to the blob data. `URL.revokeObjectURL()` should be called after the audio element has loaded to free memory. The current implementation does not appear to revoke blob URLs.

- **Caching**: Blob URLs are not cached by the HTTP cache. Each `new Audio(blobUrl)` fetches from the in-memory blob store. This is fast (no disk I/O) but means the data is held in memory until revoked.

- **No format issues with WAV**: Standard 16-bit PCM WAV over blob URLs works reliably in WebKitGTK. The GStreamer `wavparse` element handles the format without issues.

## 5. Audio Playback Best Practices for WebKit2GTK on Linux/Fedora

### Recommended Approach (Current Implementation)

The current approach is correct:

1. **Use HTMLAudioElement, not Web Audio API** for playback of pre-rendered audio (TTS output)
2. **Blob URLs** from base64-decoded WAV data
3. **Wait for `oncanplaythrough`** before calling `play()`
4. **Queue-based sequential playback** with `onended` chaining
5. **50ms silence prepend** on the Rust side to absorb PipeWire wake-up latency

### Additional Recommendations

1. **Revoke blob URLs**: Call `URL.revokeObjectURL(url)` in the `onended` handler to prevent memory leaks.

2. **Pre-warm the audio pipeline**: Play a silent audio clip (1ms of zeros) at application startup or on the first user interaction to initialize the GStreamer pipeline and wake PipeWire before real audio needs to play.

3. **Disable PipeWire suspend**: For the desktop app, consider documenting the WirePlumber configuration above, or pre-warming the pipeline frequently enough to prevent suspend.

4. **Set autoplay policy**: The current `set_media_playback_requires_user_gesture(false)` is correct. The `AutoplayPolicy` enum also exists (`WEBKIT_AUTOPLAY_ALLOW` = 0) but the per-setting approach is sufficient.

5. **Consider explicit WebAudio enable**: While enabled by default, explicitly calling `settings.set_enable_webaudio(true)` would make the configuration self-documenting and protect against future default changes if Web Audio API is ever needed again.

6. **Web Audio API is viable for synthesis (oscillators/tones)**: The `playAlertTone()` function using `OscillatorNode` likely works because it does not involve `decodeAudioData`. Oscillator + GainNode playback is a simpler pipeline that does not require GStreamer decoding.

### What NOT To Do

- **Do not use Web Audio API `decodeAudioData()`** for audio playback in this WebKit2GTK app. The GStreamer backend has known issues with this path.
- **Do not rely on `tokio::time::sleep()` delays** to work around audio clipping. The timing is unreliable and system-dependent.
- **Do not use custom URI schemes** for audio sources. Use blob URLs or HTTP URLs.

## 6. Summary Table

| Feature | Status in WebKit2GTK 2.50.5 | Notes |
|---------|----------------------------|-------|
| `AudioContext` creation | Works | Starts suspended without user gesture |
| `AudioContext.resume()` | Works | Requires prior user gesture in strict mode |
| `decodeAudioData()` (WAV) | **Unreliable** | Silently fails in this project's setup |
| `AudioBufferSourceNode.start()` | Unknown | Never reached due to decode failure |
| `OscillatorNode` | Works | Used for alert tones |
| `HTMLAudioElement` + blob URL | **Works** | Current production approach |
| `oncanplaythrough` | Fires reliably | For blob URLs with WAV data |
| `onended` | Fires reliably | Used for queue chaining |
| Programmatic `play()` | Works | With `media_playback_requires_user_gesture = false` |
| WebAudio enabled by default | Yes | `enable-webaudio` property defaults to TRUE |

## Sources

- [WebKit Bug #105298 -- GStreamer decode-audio-data-basic.html fails](https://bugs.webkit.org/show_bug.cgi?id=105298)
- [WebKit Bug #106658 -- decodeAudioData DOM Exception 12](https://bugs.webkit.org/show_bug.cgi?id=106658)
- [WebKit Bug #221334 -- WebAudio delayed and glitchy on Safari](https://bugs.webkit.org/show_bug.cgi?id=221334)
- [WebKit Bug #226922 -- Safari 15 breaks WebM Opus in Web Audio](https://bugs.webkit.org/show_bug.cgi?id=226922)
- [WebKit Bug #238113 -- Audio playback ends early with blob URL](https://bugs.webkit.org/show_bug.cgi?id=238113)
- [WebKit Bug #154538 -- Web Audio distortion after sample rate change](https://bugs.webkit.org/show_bug.cgi?id=154538)
- [WebKit Bug #186933 -- WebAudioSourceProviderGStreamer for GTK/WPE](https://bugs.webkit.org/show_bug.cgi?id=186933)
- [Implementing WebAudio in WebKit with GStreamer](https://base-art.net/Articles/1/)
- [WebKitGTK Settings: enable-webaudio](https://webkitgtk.org/reference/webkit2gtk/stable/property.Settings.enable-webaudio.html)
- [WebKitGTK Settings: media-playback-requires-user-gesture](https://webkitgtk.org/reference/webkit2gtk/2.40.3/property.Settings.media-playback-requires-user-gesture.html)
- [WebKitGTK AutoplayPolicy enum](https://webkitgtk.org/reference/webkit2gtk/stable/enum.AutoplayPolicy.html)
- [Web Audio API 1.0 Implementation Report](https://webaudio.github.io/web-audio-api/implementation-report.html)
- [Tauri #8654 -- Can't play local audio on Linux](https://github.com/tauri-apps/tauri/issues/8654)
- [Tauri #9326 -- Can't play MP3 on Linux](https://github.com/tauri-apps/tauri/issues/9326)
- [Debian Bug #866503 -- webkit2gtk stuttering audio](https://debian-bugs-dist.debian.narkive.com/ugnTTU9w/bug-866503-webkit2gtk-stuttering-audio-playback)
- [WebKitGTK 2.50.0 release](https://webkitgtk.org/2025/09/17/webkitgtk2.50.0-released.html)
- [WebKitGTK 2.50.6 release](https://webkitgtk.org/2026/03/12/webkitgtk2.50.6-released.html)
- [Fedora: Disable PipeWire suspend-on-idle](https://discussion.fedoraproject.org/t/how-do-i-disable-audio-sink-suspend-on-idle-using-wireplumber-and-pipewire-on-fedora-35-so-that-my-audio-isnt-delayed-when-playback-resumes/69861)
- [Disable WirePlumber/PipeWire suspend-on-idle (pops, delays, noise)](https://davejansen.com/disable-wireplumber-pipewire-suspend-on-idle-pops-delays-noise/)
- [GStreamer wavparse documentation](https://gstreamer.freedesktop.org/documentation/wavparse/index.html)
- [webkit2gtk-rs (Tauri)](https://github.com/tauri-apps/webkit2gtk-rs)
- [webkit2gtk Rust crate SettingsExt](https://docs.rs/webkit2gtk/latest/webkit2gtk/trait.SettingsExt.html)
- [WebKit AudioFileReaderGStreamer source](https://github.com/WebKit/webkit/tree/main/Source/WebCore/platform/audio/gstreamer)
