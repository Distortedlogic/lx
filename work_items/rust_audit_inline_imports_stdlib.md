# Inline Imports — lx stdlib modules

Replace inline `std::` and `crate::` paths with `use` imports at the top of each file.

**Supersedes:** code_cleanup.md Task 1 (store_dispatch.rs inline imports).

---

## builtins/agent.rs

Add to imports:

```
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Mutex;
use std::path::Path;
use std::fs;
```

Replace inline paths:
- `std::sync::mpsc::Sender<LxVal>` → `Sender<LxVal>` (lines 13, 126)
- `std::sync::Mutex<std::sync::mpsc::Receiver<LxVal>>` → `Mutex<Receiver<LxVal>>` (lines 14, 125)
- `std::fs::read_to_string` → `fs::read_to_string` (line 36)
- `std::sync::mpsc::channel::<LxVal>()` → `mpsc::channel::<LxVal>()` (lines 43, 44)
- `std::path::Path::new` → `Path::new` (line 47)
- `Arc::new(std::sync::Mutex::new(...))` → `Arc::new(Mutex::new(...))` (lines 49, 70)

Also add to imports:
```
use crate::interpreter::Interpreter;
use crate::parser::parse;
use crate::source::FileId;
use crate::stdlib::helpers::extract_handle_id;
```

Replace:
- `crate::interpreter::Interpreter` inline → `Interpreter` (line 59)
- `crate::parser::parse` inline → `parse` (line 39)
- `crate::source::FileId` inline → `FileId` (line 39)
- `crate::stdlib::helpers::extract_handle_id` inline → `extract_handle_id` (line 21)

---

## stdlib/fs.rs

Add to imports:

```
use std::fs;
use std::path::Path;
```

Replace all `std::fs::*` and `std::path::Path` inline calls with short names throughout the file (lines 30, 36, 43, 49, 54, 55, 61, 65, 70, 74, 79, 84, 101).

Also add `use crate::sym::Sym;` and replace `crate::sym::Sym` → `Sym` at line 13.

---

## stdlib/checkpoint.rs

Add to imports:

```
use std::fs;
use std::path::Path;
```

Replace `std::fs::*` inline calls (lines 26, 31, 32, 38, 45).

---

## stdlib/env.rs

Add to imports:

```
use std::env;
```

Replace `std::env::*` inline calls (lines 23, 32, 40, 46, 54).

---

## stdlib/store/mod.rs

Add to imports:

```
use std::fs;
use std::path::Path;
use serde_json;
```

Replace inline paths (lines 59, 60, 61, 64, 65, 68).

---

## stdlib/store/store_dispatch.rs

Add to imports:

```
use std::fs;
use indexmap::IndexMap;
use serde_json;
```

Replace inline paths at lines 42, 132, 133, 134, 141, 142, 155.

---

## stdlib/time.rs

Add to imports:

```
use std::thread;
use std::time::Duration;
```

Replace `std::thread::sleep(std::time::Duration::from_millis(ms))` → `thread::sleep(Duration::from_millis(ms))` (line 46).

---

## stdlib/channel.rs

Add to imports:

```
use std::pin::Pin;
use std::future::Future;
```

Replace inline `std::pin::Pin<Box<dyn std::future::Future<...>>>` (lines 55, 73).

---

## stdlib/cron/cron_helpers.rs

Add to imports:

```
use std::sync::LazyLock;
use std::thread;
```

Replace `std::sync::LazyLock` (line 21) and `std::thread::sleep` (line 67).

---

## stdlib/math.rs

Add to imports:

```
use std::f64::consts::{PI, E};
```

Replace `std::f64::consts::PI` → `PI`, `std::f64::consts::E` → `E` (lines 24-25).

---

## stdlib/wasm.rs

Add to imports:

```
use std::fs;
use std::env;
```

Replace `std::fs::read_to_string` (line 46), `std::env::var` (line 86).

---

## stdlib/md/md_render.rs

Add to imports:

```
use indexmap::IndexMap;
```

Replace `indexmap::IndexMap<crate::sym::Sym, LxVal>` → `IndexMap<Sym, LxVal>` (lines 55, 60, 65). Also add `use crate::sym::Sym;` if not already present.

---

## stdlib/diag/echart.rs

Add to imports:

```
use serde_json::{self, json, Value as JsonValue};
```

Replace all `serde_json::json!` → `json!`, `serde_json::Value` → `JsonValue` throughout (lines 66, 78, 86, 87, 99, 110, 116, 121).

---

## stdlib/helpers.rs

Add `use indexmap::IndexMap;` and replace inline usage (line 48).

---

## stdlib/diag/mod.rs

Add `use crate::source::FileId;` and replace `crate::source::FileId` → `FileId` (line 73).
