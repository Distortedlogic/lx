# Unit 3: Fix RSX class attribute violations

## Violation

Rule: "No string interpolation mixed with static classes" from the RSX Class Attributes section of the Dioxus audit rules.

Mixed class attributes combine string interpolation (`{variable}`) with static Tailwind classes in a single `class:` string. These must be split into multiple `class:` attributes: one for static classes and separate ones for each interpolated value.

## Full violation inventory

There are exactly 4 violations, all in `crates/lx-desktop/src/`. The entire `crates/lx-mobile/src/` tree was scanned and has zero violations.

---

### Violation 1

File: `crates/lx-desktop/src/pages/settings/quotas.rs`
Line: 53

Current:
```rust
                    div { class: "h-full {color} rounded-full", style: "{width}" }
```

Replace with:
```rust
                    div { class: "h-full rounded-full", class: "{color}", style: "{width}" }
```

Static classes: `h-full rounded-full`
Dynamic class: `{color}` (resolves to `bg-[var(--error)]`, `bg-[var(--warning)]`, or `bg-[var(--primary)]`)

---

### Violation 2

File: `crates/lx-desktop/src/pages/agents/voice_banner.rs`
Line: 140

Current:
```rust
      div { class: "bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0 {bar_glow}",
```

Replace with:
```rust
      div { class: "bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0", class: "{bar_glow}",
```

Static classes: `bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0`
Dynamic class: `{bar_glow}` (resolves to `shadow-[0_0_12px_var(--primary)]` or `""`)

---

### Violation 3

File: `crates/lx-desktop/src/pages/agents/pane_area.rs`
Line: 160

Current:
```rust
      class: "group absolute flex flex-col border {border}",
```

Replace with:
```rust
      class: "group absolute flex flex-col border",
      class: "{border}",
```

Static classes: `group absolute flex flex-col border`
Dynamic class: `{border}` (resolves to `border-[var(--primary)]` or `border-[var(--outline-variant)]/30`)

---

### Violation 4

File: `crates/lx-desktop/src/terminal/tab_bar.rs`
Line: 68

Current:
```rust
                  span { class: "text-xs {icon_color}", "{tab_icon}" }
```

Replace with:
```rust
                  span { class: "text-xs", class: "{icon_color}", "{tab_icon}" }
```

Static classes: `text-xs`
Dynamic class: `{icon_color}` (resolves to `text-[var(--primary)]` or `text-[var(--outline)]`)

---

## Verification

After making all 4 changes, run `just diagnose` and confirm no errors in the modified files. The changes are purely cosmetic at the RSX level (multiple `class:` attributes are merged by Dioxus at runtime), so behavior is unchanged.
