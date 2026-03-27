# Inline Imports — lx-cli crate

Replace inline `std::` paths with `use` imports at the top of each file. Nearly every file in lx-cli uses `std::fs`, `std::env`, `std::path` inline.

---

## check.rs

`check.rs` already imports `std::path::{Path, PathBuf}` and `std::sync::Arc`. Add:
```
use std::io;
use std::fs;
use std::env;
```

Replace these inline paths:
- Line 18: `std::io::Error` → `io::Error` (in `FixOutcome::WriteFailed`)
- Line 31: `std::fs::write(path, &fixed_source)` → `fs::write(path, &fixed_source)`
- Line 40: `std::fs::write(path, &source)` → `fs::write(path, &source)`
- Line 77: `std::fs::write(path, &fixed_source)` → `fs::write(path, &fixed_source)`
- Line 85: `std::fs::write(path, source)` → `fs::write(path, source)`
- Line 98: `std::env::current_dir()` → `env::current_dir()`
- Line 276: `std::fs::read_dir(dir)` → `fs::read_dir(dir)`

---

## main.rs

Add:
```
use std::env;
use std::fs;
use std::path::Path;
```
Replace all inline `std::env::*`, `std::fs::*`, `std::path::*` (lines 91, 141, 145, 164, 173, 198, 245).

---

## plugin.rs

Add:
```
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
```
Replace all inline paths (lines 20, 36, 51, 52, 56, 59, 63, 70, 96, 105, 123, 142, 178, 198, 202, 232).

---

## init.rs

Add:
```
use std::fs;
use std::path::Path;
```
Replace all inline paths (lines 8, 14, 37, 43, 49, 55, 61, 68, 74).

---

## testing.rs

Add:
```
use std::fs;
use std::path::{Path, PathBuf};
```
Replace all inline paths (lines 12, 26, 95, 134, 182).

---

## install.rs

Add:
```
use std::fs;
use std::path::{Path, PathBuf};
```
Replace all inline paths (lines 9, 37, 76, 89, 180).

---

## install_ops.rs

Add:
```
use std::fs;
use std::path::{Path, PathBuf};
```
Replace all inline paths (lines 48, 49, 60, 61, 69, 101, 140).

---

## manifest.rs

Add:
```
use std::fs;
use std::path::Path;
```
Replace all inline paths (lines 110, 144, 156, 160, 200, 214, 225, 242).

---

## listing.rs

Add:
```
use std::fs;
use std::path::PathBuf;
```
Replace all inline paths (lines 7, 53, 66).

---

## lockfile.rs

Add:
```
use std::fs;
use std::path::Path;
```
Replace all inline paths (lines 24, 31).

---

## agent_cmd.rs

Add:
```
use std::fs;
use std::path::Path;
```
Replace all inline paths (lines 7, 51, 52, 73).

---

## run.rs

Add:
```
use std::fs;
```
Replace inline path (line 35).

---

## Note

Some files may already import parts of `std::path` or `std::fs`. Before adding imports, verify no duplicates.
