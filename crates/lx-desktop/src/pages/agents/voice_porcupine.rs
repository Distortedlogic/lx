use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{LazyLock, Mutex};

use dioxus::logger::tracing::{error, info, warn};

type PvPorcupineT = std::ffi::c_void;

struct PorcupineLib {
  _lib: libloading::Library,
  init: unsafe extern "C" fn(*const c_char, *const c_char, i32, *const *const c_char, *const f32, *mut *mut PvPorcupineT) -> i32,
  delete: unsafe extern "C" fn(*mut PvPorcupineT),
  process: unsafe extern "C" fn(*mut PvPorcupineT, *const i16, *mut i32) -> i32,
  frame_length: unsafe extern "C" fn() -> i32,
}

unsafe impl Send for PorcupineLib {}
unsafe impl Sync for PorcupineLib {}

struct PorcupineEngine {
  lib: PorcupineLib,
  handle: *mut PvPorcupineT,
  frame_len: usize,
}

unsafe impl Send for PorcupineEngine {}
unsafe impl Sync for PorcupineEngine {}

impl Drop for PorcupineEngine {
  fn drop(&mut self) {
    unsafe { (self.lib.delete)(self.handle) };
  }
}

impl PorcupineEngine {
  fn process(&self, frame: &[i16]) -> Option<i32> {
    let mut keyword_index: i32 = -1;
    let status = unsafe { (self.lib.process)(self.handle, frame.as_ptr(), &mut keyword_index) };
    if status != 0 {
      return None;
    }
    if keyword_index >= 0 { Some(keyword_index) } else { None }
  }
}

static ENGINE: LazyLock<Option<PorcupineEngine>> = LazyLock::new(|| {
  let access_key = std::env::var("PICOVOICE_ACCESS_KEY").ok()?;
  let model_path = std::env::var("PICOVOICE_MODEL_PATH").ok()?;
  let keyword_path = std::env::var("PICOVOICE_KEYWORD_PATH").ok()?;
  let lib_path = std::env::var("PICOVOICE_LIBRARY_PATH").unwrap_or_else(|_| "libpv_porcupine.so".into());

  let lib = match unsafe { libloading::Library::new(&lib_path) } {
    Ok(l) => l,
    Err(e) => {
      warn!("porcupine: failed to load {lib_path}: {e}");
      return None;
    },
  };

  let (init_fn, delete_fn, process_fn, frame_length_fn) = unsafe {
    let init = *lib
      .get::<unsafe extern "C" fn(*const c_char, *const c_char, i32, *const *const c_char, *const f32, *mut *mut PvPorcupineT) -> i32>(b"pv_porcupine_init\0")
      .ok()?;
    let delete = *lib.get::<unsafe extern "C" fn(*mut PvPorcupineT)>(b"pv_porcupine_delete\0").ok()?;
    let process = *lib.get::<unsafe extern "C" fn(*mut PvPorcupineT, *const i16, *mut i32) -> i32>(b"pv_porcupine_process\0").ok()?;
    let frame_length = *lib.get::<unsafe extern "C" fn() -> i32>(b"pv_porcupine_frame_length\0").ok()?;
    (init, delete, process, frame_length)
  };

  let porcupine_lib = PorcupineLib { _lib: lib, init: init_fn, delete: delete_fn, process: process_fn, frame_length: frame_length_fn };
  let frame_len = unsafe { (porcupine_lib.frame_length)() } as usize;

  let c_access_key = CString::new(access_key).ok()?;
  let c_model_path = CString::new(model_path).ok()?;
  let c_keyword_path = CString::new(keyword_path).ok()?;
  let keyword_paths = [c_keyword_path.as_ptr()];
  let sensitivities = [0.5f32];
  let mut handle: *mut PvPorcupineT = std::ptr::null_mut();

  let status = unsafe { (porcupine_lib.init)(c_access_key.as_ptr(), c_model_path.as_ptr(), 1, keyword_paths.as_ptr(), sensitivities.as_ptr(), &mut handle) };
  if status != 0 || handle.is_null() {
    error!("porcupine: init failed with status {status}");
    return None;
  }

  info!("porcupine: initialized, frame_length={frame_len}");
  Some(PorcupineEngine { lib: porcupine_lib, handle, frame_len })
});

static FRAME_BUFFER: LazyLock<Mutex<Vec<i16>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn is_available() -> bool {
  ENGINE.is_some()
}

pub fn feed_samples(samples: &[i16]) -> Option<i32> {
  let engine = ENGINE.as_ref()?;
  let mut buf = FRAME_BUFFER.lock().expect("lock poisoned");
  buf.extend_from_slice(samples);
  let mut detected = None;
  while buf.len() >= engine.frame_len {
    let frame: Vec<i16> = buf.drain(..engine.frame_len).collect();
    if let Some(idx) = engine.process(&frame) {
      detected = Some(idx);
    }
  }
  detected
}

pub fn reset_buffer() {
  FRAME_BUFFER.lock().expect("lock poisoned").clear();
}
