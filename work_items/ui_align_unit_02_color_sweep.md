# UI Alignment Unit 02: Hardcoded Color Sweep

## Goal

Replace every hardcoded Tailwind color class (`gray-*`, `white`, `blue-*`, `amber-*`, `emerald-*`, `cyan-*`, `green-*`, `yellow-*`, `red-*`, `neutral-*`, `sky-*`) with the corresponding Material Design CSS variable reference across all listed component and page files.

---

## File 1: `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/sidebar.rs`

### Change 1a (line 8)
**old_string:** `aside { class: "w-60 h-full min-h-0 border-r border-gray-700/50 bg-[var(--surface-container-lowest)] flex flex-col",`
**new_string:** `aside { class: "w-60 h-full min-h-0 border-r border-[var(--outline-variant)]/50 bg-[var(--surface-container-lowest)] flex flex-col",`

### Change 1b (line 10)
**old_string:** `span { class: "flex-1 text-sm font-bold text-white truncate pl-1", "lx workspace" }`
**new_string:** `span { class: "flex-1 text-sm font-bold text-[var(--on-surface)] truncate pl-1", "lx workspace" }`

### Change 1c (line 86)
**old_string:** `div { class: "px-3 py-1.5 text-[10px] font-medium uppercase tracking-widest font-mono text-gray-500",`
**new_string:** `div { class: "px-3 py-1.5 text-[10px] font-medium uppercase tracking-widest font-mono text-[var(--outline)]",`

### Change 1d (line 99)
**old_string:** `active_class: "bg-white/10 text-white",`
**new_string:** `active_class: "bg-[var(--on-surface)]/10 text-[var(--on-surface)]",`

### Change 1e (line 100)
**old_string:** `class: "flex items-center gap-2.5 px-3 py-2 text-[13px] font-medium transition-colors text-gray-400 hover:bg-white/5 hover:text-white",`
**new_string:** `class: "flex items-center gap-2.5 px-3 py-2 text-[13px] font-medium transition-colors text-[var(--on-surface-variant)] hover:bg-[var(--on-surface)]/5 hover:text-[var(--on-surface)]",`

---

## File 2: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/comment_thread.rs`

### Change 2a (line 35)
**old_string:** `p { class: "text-sm text-gray-400", "No comments yet." }`
**new_string:** `p { class: "text-sm text-[var(--on-surface-variant)]", "No comments yet." }`

### Change 2b (line 39)
**old_string:** `div { class: "border border-gray-700 p-3 overflow-hidden min-w-0 rounded-sm",`
**new_string:** `div { class: "border border-[var(--outline-variant)] p-3 overflow-hidden min-w-0 rounded-sm",`

### Change 2c (line 45)
**old_string:** `span { class: "text-xs text-gray-400", "{comment.created_at}" }`
**new_string:** `span { class: "text-xs text-[var(--on-surface-variant)]", "{comment.created_at}" }`

### Change 2d (line 56)
**old_string:** `class: "w-full bg-gray-800 border border-gray-600 rounded p-2 text-sm outline-none resize-none min-h-[60px] placeholder:text-gray-500",`
**new_string:** `class: "w-full bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded p-2 text-sm outline-none resize-none min-h-[60px] placeholder:text-[var(--outline)]",`

### Change 2e (line 63)
**old_string:** `class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded transition-colors disabled:opacity-50",`
**new_string:** `class: "px-3 py-1.5 bg-[var(--primary)] hover:brightness-110 text-[var(--on-primary)] text-sm rounded transition-colors disabled:opacity-50",`

---

## File 3: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/filter_bar.rs`

### Change 3a (line 19)
**old_string:** `span { class: "inline-flex items-center gap-1 rounded-full bg-gray-700 px-2.5 py-0.5 text-xs pr-1",`
**new_string:** `span { class: "inline-flex items-center gap-1 rounded-full bg-[var(--surface-container-high)] px-2.5 py-0.5 text-xs pr-1",`

### Change 3b (line 20)
**old_string:** `span { class: "text-gray-400", "{filter.label}:" }`
**new_string:** `span { class: "text-[var(--on-surface-variant)]", "{filter.label}:" }`

### Change 3c (line 23)
**old_string:** `class: "ml-1 rounded-full hover:bg-gray-600 p-0.5",`
**new_string:** `class: "ml-1 rounded-full hover:bg-[var(--surface-bright)] p-0.5",`

### Change 3d (line 33)
**old_string:** `class: "text-xs text-gray-400 hover:text-white px-2 py-1 transition-colors",`
**new_string:** `class: "text-xs text-[var(--on-surface-variant)] hover:text-[var(--on-surface)] px-2 py-1 transition-colors",`

---

## File 4: `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/mod.rs`

### Change 4a (line 71)
**old_string:** `h3 { class: "text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3",`
**new_string:** `h3 { class: "text-sm font-semibold text-[var(--on-surface-variant)] uppercase tracking-wide mb-3",`

### Change 4b (line 74)
**old_string:** `div { class: "border border-gray-700 divide-y divide-gray-700 overflow-hidden",`
**new_string:** `div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",`

### Change 4c (line 76)
**old_string:** `div { class: "px-4 py-2.5 text-sm hover:bg-white/5 transition-colors",`
**new_string:** `div { class: "px-4 py-2.5 text-sm hover:bg-[var(--on-surface)]/5 transition-colors",`

### Change 4d (line 79)
**old_string:** `span { class: "text-gray-400 font-mono text-xs",`
**new_string:** `span { class: "text-[var(--on-surface-variant)] font-mono text-xs",`

### Change 4e (line 84)
**old_string:** `span { class: "text-xs text-gray-500 shrink-0", "{event.timestamp}" }`
**new_string:** `span { class: "text-xs text-[var(--outline)] shrink-0", "{event.timestamp}" }`

---

## File 5: `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/activity_charts.rs`

### Change 5a (line 10)
**old_string:** `div { class: "border border-gray-700 rounded-lg p-4 space-y-3",`
**new_string:** `div { class: "border border-[var(--outline-variant)] rounded-lg p-4 space-y-3",`

### Change 5b (line 12)
**old_string:** `h3 { class: "text-xs font-medium text-gray-400", "{title}" }`
**new_string:** `h3 { class: "text-xs font-medium text-[var(--on-surface-variant)]", "{title}" }`

### Change 5c (line 14)
**old_string:** `span { class: "text-[10px] text-gray-500", "{sub}" }`
**new_string:** `span { class: "text-[10px] text-[var(--outline)]", "{sub}" }`

### Change 5d (line 28)
**old_string:** `class: "flex-1 bg-gray-700/30 rounded-sm",`
**new_string:** `class: "flex-1 bg-[var(--outline-variant)]/30 rounded-sm",`

### Change 5e (line 42)
**old_string:** `class: "flex-1 bg-emerald-700/30 rounded-sm",`
**new_string:** `class: "flex-1 bg-[var(--primary)]/30 rounded-sm",`

---

## File 6: `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/dashboard/active_agents_panel.rs`

### Change 6a (line 34)
**old_string:** `h3 { class: "mb-3 text-sm font-semibold uppercase tracking-wide text-gray-400",`
**new_string:** `h3 { class: "mb-3 text-sm font-semibold uppercase tracking-wide text-[var(--on-surface-variant)]",`

### Change 6b (line 38)
**old_string:** `div { class: "rounded-xl border border-gray-700 p-4",`
**new_string:** `div { class: "rounded-xl border border-[var(--outline-variant)] p-4",`

### Change 6c (line 39)
**old_string:** `p { class: "text-sm text-gray-400", "No recent agent runs." }`
**new_string:** `p { class: "text-sm text-[var(--on-surface-variant)]", "No recent agent runs." }`

### Change 6d (line 59)
**old_string:** `div { class: "flex h-[200px] flex-col overflow-hidden rounded-xl border border-gray-700 shadow-sm bg-[var(--surface-container)]",`
**new_string:** `div { class: "flex h-[200px] flex-col overflow-hidden rounded-xl border border-[var(--outline-variant)] shadow-sm bg-[var(--surface-container)]",`

### Change 6e (line 60)
**old_string:** `div { class: "border-b border-gray-700/60 px-3 py-3",`
**new_string:** `div { class: "border-b border-[var(--outline-variant)]/60 px-3 py-3",`

### Change 6f (line 64)
**old_string:** `span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-cyan-400 opacity-70" }`
**new_string:** `span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--tertiary)] opacity-70" }`

### Change 6g (line 65)
**old_string:** `span { class: "relative inline-flex h-2.5 w-2.5 rounded-full bg-cyan-500" }`
**new_string:** `span { class: "relative inline-flex h-2.5 w-2.5 rounded-full bg-[var(--tertiary)]" }`

### Change 6h (line 68)
**old_string:** `span { class: "inline-flex h-2.5 w-2.5 rounded-full bg-gray-500" }`
**new_string:** `span { class: "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--outline)]" }`

### Change 6i (line 72)
**old_string:** `div { class: "mt-2 text-[11px] text-gray-400", "{last_seen}" }`
**new_string:** `div { class: "mt-2 text-[11px] text-[var(--on-surface-variant)]", "{last_seen}" }`

### Change 6j (line 75)
**old_string:** `p { class: "text-xs text-gray-500", "No transcript available." }`
**new_string:** `p { class: "text-xs text-[var(--outline)]", "No transcript available." }`

---

## File 7: `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/activity.rs`

### Change 7a (line 29)
**old_string:** `class: "h-8 rounded-md border border-gray-600 bg-gray-800 px-2 py-1 text-xs focus:outline-none focus:ring-1 focus:ring-blue-500",`
**new_string:** `class: "h-8 rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)] px-2 py-1 text-xs focus:outline-none focus:ring-1 focus:ring-[var(--primary)]",`

### Change 7b (line 42)
**old_string:** `div { class: "border border-gray-700 divide-y divide-gray-700 overflow-hidden",`
**new_string:** `div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",`

### Change 7c (line 44)
**old_string:** `div { class: "flex items-center px-4 py-2.5 hover:bg-white/5 transition-colors text-sm",`
**new_string:** `div { class: "flex items-center px-4 py-2.5 hover:bg-[var(--on-surface)]/5 transition-colors text-sm",`

### Change 7d (line 45)
**old_string:** `span { class: "w-40 shrink-0 text-gray-500 font-mono text-xs",`
**new_string:** `span { class: "w-40 shrink-0 text-[var(--outline)] font-mono text-xs",`

### Change 7e (line 51)
**old_string:** `span { class: "flex-1 text-gray-300 truncate", "{event.message}" }`
**new_string:** `span { class: "flex-1 text-[var(--on-surface)] truncate", "{event.message}" }`

---

## File 8: `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/goals/tree.rs`

### Change 8a (line 48)
**old_string:** `class: "flex items-center gap-2 px-2 py-2 hover:bg-white/5 transition-colors border-b border-[var(--outline-variant)]/20 last:border-b-0",`
**new_string:** `class: "flex items-center gap-2 px-2 py-2 hover:bg-[var(--on-surface)]/5 transition-colors border-b border-[var(--outline-variant)]/20 last:border-b-0",`

---

## File 9: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/inline_editor.rs`

### Change 9a (line 64)
**old_string:** `class: "cursor-pointer rounded hover:bg-white/5 transition-colors px-1 -mx-1 {extra}",`
**new_string:** `class: "cursor-pointer rounded hover:bg-[var(--on-surface)]/5 transition-colors px-1 -mx-1 {extra}",`

### Change 9b (line 67)
**old_string:** `span { class: "text-gray-400 italic", "{placeholder}" }`
**new_string:** `span { class: "text-[var(--on-surface-variant)] italic", "{placeholder}" }`

---

## File 10: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/inline_entity_selector.rs`

### Change 10a (line 29)
**old_string:** `class: "inline-flex min-w-0 items-center gap-1 rounded-md border border-gray-600 bg-gray-800/40 px-2 py-1 text-sm font-medium transition-colors hover:bg-white/5 {extra}",`
**new_string:** `class: "inline-flex min-w-0 items-center gap-1 rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)]/40 px-2 py-1 text-sm font-medium transition-colors hover:bg-[var(--on-surface)]/5 {extra}",`

### Change 10b (line 34)
**old_string:** `span { class: "text-gray-400", "{placeholder}" }`
**new_string:** `span { class: "text-[var(--on-surface-variant)]", "{placeholder}" }`

### Change 10c (line 45)
**old_string:** `div { class: "absolute z-50 mt-1 w-[min(20rem,calc(100vw-2rem))] rounded-md border border-gray-600 bg-gray-800 shadow-lg",`
**new_string:** `div { class: "absolute z-50 mt-1 w-[min(20rem,calc(100vw-2rem))] rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container-high)] shadow-lg",`

### Change 10d (line 47)
**old_string:** `class: "w-full border-b border-gray-600 bg-transparent px-2 py-1.5 text-sm outline-none placeholder:text-gray-500",`
**new_string:** `class: "w-full border-b border-[var(--outline-variant)] bg-transparent px-2 py-1.5 text-sm outline-none placeholder:text-[var(--outline)]",`

### Change 10e (line 59)
**old_string:** `class: "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-white/5",`
**new_string:** `class: "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-[var(--on-surface)]/5",`

### Change 10f (line 67)
**old_string:** `span { class: "material-symbols-outlined text-sm text-gray-400 ml-auto", "check" }`
**new_string:** `span { class: "material-symbols-outlined text-sm text-[var(--on-surface-variant)] ml-auto", "check" }`

### Change 10g (line 74)
**old_string:** `p { class: "px-2 py-2 text-xs text-gray-400", "No results." }`
**new_string:** `p { class: "px-2 py-2 text-xs text-[var(--on-surface-variant)]", "No results." }`

---

## File 11: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/page_skeleton.rs`

### Change 11a (line 6)
**old_string:** `div { class: "animate-pulse bg-gray-700/50 rounded {class}" }`
**new_string:** `div { class: "animate-pulse bg-[var(--outline-variant)]/50 rounded {class}" }`

---

## File 12: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/metric_card.rs`

### Change 12a (line 13)
**old_string:** `let hover_class = if clickable { "hover:bg-white/5 cursor-pointer" } else { "" };`
**new_string:** `let hover_class = if clickable { "hover:bg-[var(--on-surface)]/5 cursor-pointer" } else { "" };`

### Change 12b (line 25)
**old_string:** `p { class: "text-sm font-medium text-gray-400 mt-1", "{label}" }`
**new_string:** `p { class: "text-sm font-medium text-[var(--on-surface-variant)] mt-1", "{label}" }`

### Change 12c (line 27)
**old_string:** `div { class: "text-xs text-gray-500 mt-1.5", "{desc_text}" }`
**new_string:** `div { class: "text-xs text-[var(--outline)] mt-1.5", "{desc_text}" }`

### Change 12d (line 30)
**old_string:** `span { class: "material-symbols-outlined text-base text-gray-500 shrink-0 mt-1.5",`
**new_string:** `span { class: "material-symbols-outlined text-base text-[var(--outline)] shrink-0 mt-1.5",`

---

## File 13: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/page_tab_bar.rs`

### Change 13a (line 18)
**old_string:** `div { class: "flex border-b border-gray-700/50",`
**new_string:** `div { class: "flex border-b border-[var(--outline-variant)]/50",`

### Change 13b (line 22)
**old_string:** `"border-white text-white"`
**new_string:** `"border-[var(--on-surface)] text-[var(--on-surface)]"`

### Change 13c (line 24)
**old_string:** `"border-transparent text-gray-400 hover:text-white hover:border-gray-500"`
**new_string:** `"border-transparent text-[var(--on-surface-variant)] hover:text-[var(--on-surface)] hover:border-[var(--outline)]"`

---

## File 14: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/empty_state.rs`

### Change 14a (line 7)
**old_string:** `div { class: "bg-gray-800/50 p-4 mb-4",`
**new_string:** `div { class: "bg-[var(--surface-container)]/50 p-4 mb-4",`

### Change 14b (line 8)
**old_string:** `span { class: "material-symbols-outlined text-4xl text-gray-500", "{icon}" }`
**new_string:** `span { class: "material-symbols-outlined text-4xl text-[var(--outline)]", "{icon}" }`

### Change 14c (line 10)
**old_string:** `p { class: "text-sm text-gray-400 mb-4", "{message}" }`
**new_string:** `p { class: "text-sm text-[var(--on-surface-variant)] mb-4", "{message}" }`

### Change 14d (line 16)
**old_string:** `class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded transition-colors",`
**new_string:** `class: "px-4 py-2 bg-[var(--primary)] hover:brightness-110 text-[var(--on-primary)] text-sm rounded transition-colors",`

---

## File 15: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/identity.rs`

### Change 15a (line 43)
**old_string:** `span { class: "inline-flex items-center justify-center rounded-full bg-gray-700 text-gray-300 shrink-0 {avatar_size}",`
**new_string:** `span { class: "inline-flex items-center justify-center rounded-full bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] shrink-0 {avatar_size}",`

---

## File 16: `/home/entropybender/repos/lx/crates/lx-desktop/src/components/entity_row.rs`

### Change 16a (line 17)
**old_string:** `let base = "flex items-center gap-3 px-4 py-2 text-sm border-b border-gray-700/50 last:border-b-0 transition-colors";`
**new_string:** `let base = "flex items-center gap-3 px-4 py-2 text-sm border-b border-[var(--outline-variant)]/50 last:border-b-0 transition-colors";`

### Change 16b (line 18)
**old_string:** `let hover = if interactive { " cursor-pointer hover:bg-white/5" } else { "" };`
**new_string:** `let hover = if interactive { " cursor-pointer hover:bg-[var(--on-surface)]/5" } else { "" };`

### Change 16c (line 19)
**old_string:** `let sel = if selected { " bg-white/[0.03]" } else { "" };`
**new_string:** `let sel = if selected { " bg-[var(--on-surface)]/[0.03]" } else { "" };`

### Change 16d (line 29)
**old_string:** `span { class: "text-xs text-gray-400 font-mono shrink-0", "{id}" }`
**new_string:** `span { class: "text-xs text-[var(--on-surface-variant)] font-mono shrink-0", "{id}" }`

### Change 16e (line 34)
**old_string:** `p { class: "text-xs text-gray-400 truncate mt-0.5", "{sub}" }`
**new_string:** `p { class: "text-xs text-[var(--on-surface-variant)] truncate mt-0.5", "{sub}" }`

---

## Summary

Total files: 16
Total individual replacements: 43

Every `gray-*` maps to either `var(--outline-variant)` (borders, dividers), `var(--outline)` (dimmer text, timestamps), `var(--on-surface-variant)` (secondary text), or `var(--surface-container*)` (backgrounds). Every `white` maps to `var(--on-surface)`. Every `blue-*` maps to `var(--primary)` / `var(--on-primary)`. Every `cyan-*` maps to `var(--tertiary)`. Every `emerald-*` maps to `var(--primary)` (green accent).
