# Inline Imports — lx-desktop crate

Replace inline `std::` paths with `use` imports at the top of each file.

---

## pages/agents/voice_pipeline.rs

Current imports include `std::sync::LazyLock` but NOT `Arc`, `Mutex`, `Cursor`, or `PI`. Add:
```
use std::sync::{Arc, Mutex};
use std::io::Cursor;
use std::f64::consts::PI;
```

Replace inline paths:
- Line 18: `std::sync::Mutex<Option<std::sync::Arc<...>>>` → `Mutex<Option<Arc<...>>>`
- Line 28: `std::io::Cursor` → `Cursor`
- Lines 33, 34: `std::sync::Arc::new` → `Arc::new`
- Line 56: `std::f64::consts::PI` → `PI`
- Line 70: `std::sync::Arc::new` → `Arc::new`

---

## pages/agents/voice_context.rs

Add:
```
use std::fmt::{self, Display, Formatter};
```
Replace inline `std::fmt::Display`, `std::fmt::Formatter` (lines 12-13, 32-33).

---

## pages/agents/voice_porcupine.rs

Add:
```
use std::env;
use std::path::Path;
```
Replace inline paths (lines 13-14).

---

## pages/agents/voice_banner.rs

Add:
```
use std::mem;
```
Replace `std::mem::take` → `mem::take` (line 44).

---

## pages/agents/pane_area.rs

Add:
```
use std::env;
```
Replace `std::env::current_dir` → `env::current_dir` (line 16).

---

## pages/tools/mcp_panel.rs

Add:
```
use std::io;
```
Replace `std::io::Error` → `io::Error`, `std::io::ErrorKind` → `io::ErrorKind` (lines 3, 5).

---

## pages/events.rs

Add:
```
use std::collections::HashSet;
```
Replace inline `std::collections::HashSet` (lines 70, 118).

---

## layout/menu_bar.rs

Add:
```
use std::env;
```
Replace `std::env::current_dir` → `env::current_dir` (lines 21, 43, 53).

---

## contexts/activity_log.rs

Add:
```
use std::time::{SystemTime, UNIX_EPOCH};
```
Replace inline paths (line 19).

---

## build.rs

Add:
```
use std::env;
use std::fs;
```
Replace inline `std::env::var`, `std::fs::*` (lines 5, 14, 26, 51).
