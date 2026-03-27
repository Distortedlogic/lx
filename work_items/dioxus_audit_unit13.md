# Unit 13: Inline single-use settings components

## Problem

`ArchitectCard` (34 lines) and `SystemNotice` (16 lines) in `crates/lx-desktop/src/pages/settings/task_priority.rs` are `#[component]` functions rendered at exactly one call site each (`settings/mod.rs` lines 42-43). They have no hooks, no state, and no conditional logic — they are static RSX. Per the audit rule, single-use components should be inlined into the parent.

## Scope

Only `ArchitectCard` and `SystemNotice` are inlined. The other settings panels (`EnvVarsPanel`, `QuotasPanel`, `TaskPriorityPanel`) have hooks and state — inlining all of them into `mod.rs` would create a 260+ line component which is close to the 300-line limit and loses modularity. Those panels stay as separate components.

## Files

| File | Change |
|------|--------|
| `crates/lx-desktop/src/pages/settings/task_priority.rs` | Remove `ArchitectCard` and `SystemNotice` definitions + their `pub` exports |
| `crates/lx-desktop/src/pages/settings/mod.rs` | Replace `ArchitectCard {}` and `SystemNotice {}` with their inline RSX bodies |

## Dependency

Execute AFTER Unit 12 (SettingsState Store migration). Unit 12 changes access patterns in `task_priority.rs` and `mod.rs`. Running this unit first would create merge conflicts.

## Tasks

### 1. Update `crates/lx-desktop/src/pages/settings/mod.rs`

**Line 11**: Remove the import of `ArchitectCard` and `SystemNotice`:
```rust
// OLD: use self::task_priority::{ArchitectCard, SystemNotice, TaskPriorityPanel};
// NEW: use self::task_priority::TaskPriorityPanel;
```

**Lines 42-43**: Replace component invocations with inline RSX:
```rust
// OLD:
ArchitectCard {}
SystemNotice {}

// NEW — inline ArchitectCard body:
div { class: "relative bg-[var(--surface-container-lowest)] border-2 border-white p-4",
  span { class: "absolute -top-3 -right-3 bg-[var(--warning)] text-black text-[10px] px-2 py-1 font-black uppercase tracking-wider",
    "LIVE"
  }
  div { class: "flex items-center gap-4 mb-4",
    div { class: "w-12 h-12 border-2 border-[var(--outline)] p-1",
      div { class: "w-full h-full bg-[var(--warning)] flex items-center justify-center",
        span { class: "material-symbols-outlined text-black font-bold",
          "smart_toy"
        }
      }
    }
    div {
      span { class: "text-sm font-bold uppercase tracking-wider text-[var(--on-surface)]",
        "ARCHITECT_01"
      }
      p { class: "text-[10px] text-[var(--on-surface-variant)] uppercase font-mono",
        "ID: 948-XFF-001"
      }
    }
  }
  div { class: "pt-2 border-t border-[var(--outline-variant)]" }
  div { class: "flex justify-between text-xs mb-1 pt-2",
    span { class: "text-[var(--outline)] uppercase tracking-wider", "RUNTIME" }
    span { class: "text-[var(--on-surface-variant)]", "284:12:05" }
  }
  div { class: "flex justify-between text-xs",
    span { class: "text-[var(--outline)] uppercase tracking-wider", "LOAD_FACTOR" }
    span { class: "text-[var(--primary)] font-mono", "OPTIMAL" }
  }
}
// NEW — inline SystemNotice body:
div { class: "bg-[var(--surface-container)] p-4 border-l-4 border-[var(--tertiary)]",
  div { class: "flex items-start gap-3",
    span { class: "material-symbols-outlined text-[var(--tertiary)] text-lg",
      "info"
    }
    p { class: "text-[10px] text-[var(--on-surface-variant)] leading-relaxed",
      span { class: "text-white font-bold", "SYSTEM_NOTICE: " }
      "All configuration changes require manual validation before persisting to the blockchain ledger. Expect a 120ms latency injection during the verification cycle."
    }
  }
}
```

### 2. Update `crates/lx-desktop/src/pages/settings/task_priority.rs`

Remove lines 70-123 (the `ArchitectCard` and `SystemNotice` component definitions). The file should end after `TaskPriorityPanel`'s closing brace at line 69.

Remove the `pub` visibility from `ArchitectCard` and `SystemNotice` — actually, just delete the entire definitions since they're no longer used.

## Line Count Verification

After changes:
- `mod.rs`: was 48 lines → ~98 lines (48 + ~50 inline RSX). Under 300. ✓
- `task_priority.rs`: was 123 lines → ~69 lines (removed 54 lines of ArchitectCard + SystemNotice). Under 300. ✓

## Verification

`just diagnose` must pass with zero warnings.
