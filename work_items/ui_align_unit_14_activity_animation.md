# UNIT 14: Activity Row Entry Animation

## Goal

Add a fade-in + background highlight animation to activity rows in both the Activity page
and the Dashboard "Recent Activity" section.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/tailwind.css` | Edit (add keyframes + class) |
| `crates/lx-desktop/src/pages/activity.rs` | Edit (add animation class + key) |
| `crates/lx-desktop/src/pages/dashboard/mod.rs` | Edit (add animation class + key) |

## Current State

### tailwind.css (75 lines)

Contains theme variables, scrollbar styles, and two utility classes (`.shadow-ambient`, `.border-ghost`).
No `@keyframes` rules exist.

### activity.rs (58 lines)

Activity rows rendered at lines 43-53:
```rust
        div { class: "border border-gray-700 divide-y divide-gray-700 overflow-hidden",
          for event in filtered.iter() {
            div { class: "flex items-center px-4 py-2.5 hover:bg-white/5 transition-colors text-sm",
              span { class: "w-40 shrink-0 text-gray-500 font-mono text-xs",
                "{event.timestamp}"
              }
              span { class: "w-28 shrink-0 text-[var(--primary)] uppercase font-semibold text-xs",
                "{event.kind}"
              }
              span { class: "flex-1 text-gray-300 truncate", "{event.message}" }
            }
          }
        }
```

No `key` attribute on the row div. No animation classes.

### dashboard/mod.rs (100 lines)

Dashboard "Recent Activity" rows at lines 74-88:
```rust
      div { class: "min-w-0",
        h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3",
          "Recent Activity"
        }
        div { class: "border border-gray-700 divide-y divide-gray-700 overflow-hidden",
          for event in events.iter().take(10) {
            div { class: "px-4 py-2.5 text-sm hover:bg-white/5 transition-colors",
              div { class: "flex gap-3",
                p { class: "flex-1 min-w-0 truncate",
                  span { class: "text-gray-400 font-mono text-xs",
                    "{event.kind}"
                  }
                  span { class: "ml-2", "{event.message}" }
                }
                span { class: "text-xs text-gray-500 shrink-0", "{event.timestamp}" }
              }
            }
          }
        }
      }
```

No `key` attribute. No animation classes.

Depends on: Unit 02 (color sweep), Unit 04 (dialog fade-in animation).

## Step 1: Add CSS animation to `crates/lx-desktop/src/tailwind.css`

Append at the end of the file, after Unit 04's additions:

Find:
```css
.animate-fade-in {
  animation: fade-in 150ms ease-out;
}
```

Replace with:
```css
.animate-fade-in {
  animation: fade-in 150ms ease-out;
}

@keyframes activity-row-enter {
  0% {
    opacity: 0;
    background-color: rgba(156, 255, 147, 0.08);
  }
  40% {
    opacity: 1;
    background-color: rgba(156, 255, 147, 0.06);
  }
  100% {
    opacity: 1;
    background-color: transparent;
  }
}

.animate-activity-enter {
  animation: activity-row-enter 800ms ease-out both;
}
```

The rgb values `156, 255, 147` match `--primary: #9cff93`.

## Step 2: Add animation class and key to activity rows in `crates/lx-desktop/src/pages/activity.rs`

Find:
```rust
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in filtered.iter() {
            div { class: "flex items-center px-4 py-2.5 hover:bg-[var(--on-surface)]/5 transition-colors text-sm",
              span { class: "w-40 shrink-0 text-[var(--outline)] font-mono text-xs",
                "{event.timestamp}"
              }
              span { class: "w-28 shrink-0 text-[var(--primary)] uppercase font-semibold text-xs",
                "{event.kind}"
              }
              span { class: "flex-1 text-[var(--on-surface)] truncate", "{event.message}" }
            }
          }
        }
```

Replace with:
```rust
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in filtered.iter() {
            div {
              key: "{event.timestamp}-{event.kind}-{event.message}",
              class: "flex items-center px-4 py-2.5 hover:bg-[var(--on-surface)]/5 transition-colors text-sm animate-activity-enter",
              span { class: "w-40 shrink-0 text-[var(--outline)] font-mono text-xs",
                "{event.timestamp}"
              }
              span { class: "w-28 shrink-0 text-[var(--primary)] uppercase font-semibold text-xs",
                "{event.kind}"
              }
              span { class: "flex-1 text-[var(--on-surface)] truncate", "{event.message}" }
            }
          }
        }
```

Changes:
- Added `key: "{event.timestamp}-{event.kind}-{event.message}"` -- stable key ensures the animation only plays when a new row mounts, not on re-renders
- Added `animate-activity-enter` to the class string

## Step 3: Add animation class and key to dashboard activity rows in `crates/lx-desktop/src/pages/dashboard/mod.rs`

Find:
```rust
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in events.iter().take(10) {
            div { class: "px-4 py-2.5 text-sm hover:bg-[var(--on-surface)]/5 transition-colors",
              div { class: "flex gap-3",
                p { class: "flex-1 min-w-0 truncate",
                  span { class: "text-[var(--on-surface-variant)] font-mono text-xs",
                    "{event.kind}"
                  }
                  span { class: "ml-2", "{event.message}" }
                }
                span { class: "text-xs text-[var(--outline)] shrink-0", "{event.timestamp}" }
              }
            }
          }
        }
```

Replace with:
```rust
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in events.iter().take(10) {
            div {
              key: "{event.timestamp}-{event.kind}-{event.message}",
              class: "px-4 py-2.5 text-sm hover:bg-[var(--on-surface)]/5 transition-colors animate-activity-enter",
              div { class: "flex gap-3",
                p { class: "flex-1 min-w-0 truncate",
                  span { class: "text-[var(--on-surface-variant)] font-mono text-xs",
                    "{event.kind}"
                  }
                  span { class: "ml-2", "{event.message}" }
                }
                span { class: "text-xs text-[var(--outline)] shrink-0", "{event.timestamp}" }
              }
            }
          }
        }
```

Changes:
- Added `key: "{event.timestamp}-{event.kind}-{event.message}"` for stable identity
- Added `animate-activity-enter` to the class string

## Animation Behavior

- Each row fades from `opacity: 0` to `opacity: 1` over 800ms
- Background starts with a subtle green tint (`--primary` at 8% opacity) that fades to transparent
- `animation-fill-mode: both` (via `both` in the shorthand) ensures the element starts invisible
- The `key` attribute ensures Dioxus treats each row as a unique DOM element -- the animation plays once when the element first mounts and does not replay on re-renders of unchanged rows

## Verification

Run `just diagnose` and confirm no compiler errors in `crates/lx-desktop`.
