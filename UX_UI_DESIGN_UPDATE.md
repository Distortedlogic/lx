# UX/UI Design Update: Industrial Console

## Goal

Restyle lx-desktop to match the Industrial Console design system. Replace all hardcoded Tailwind gray/blue classes with CSS custom properties. Eliminate all explicit borders. Add missing structural components.

## Design System Reference

### Colors
- Surfaces: `--surface` #131313, `--surface-container-low` #1A1A1A, `--surface-container` #201F1F, `--surface-container-high` #2A2A2A, `--surface-container-highest` #353534, `--surface-container-lowest` #0E0E0E, `--surface-bright` #3A3939
- Primary: `--primary` #FFB87B, `--primary-container` #FF8F00, `--on-primary` #131313
- Text: `--on-surface` #E6E1DD, `--on-surface-variant` #DCC1AE
- Outline: `--outline` #A48C7A, `--outline-variant` #564334
- Tertiary: `--tertiary` #87CFFF
- Status: `--error` #EF5350, `--warning` #FFB300, `--success` #66BB6A

### Rules
- **No-Line Rule:** No 1px solid borders for sectioning. Use tonal shifts.
- **Ghost Border Fallback:** `outline-variant` at 15% opacity only when required for accessibility.
- **Glassmorphism:** Floating panels use `surface-container-high` at 80% opacity + `backdrop-blur-[12px]`.
- **Ambient Shadows:** Large soft blur (20-40px) at 6% opacity, amber-tinted charcoal, never pure black.
- **Primary (Amber):** Use sparingly — represents "Power On" or "Critical Focus."

### Typography
- Display/Headlines: Space Grotesk (500, 700)
- Body/Labels: Inter (400, 500, 600)
- Labels: tracked out (`letter-spacing: 0.05em`), uppercase
- Terminal/Code: high-legibility mono font

---

## Task List

### T1. Define CSS variables in `tailwind.css`

Add all design system tokens as custom properties on `:root`:
- Surfaces: `--surface` #131313, `--surface-container-low` #1A1A1A, `--surface-container` #201F1F, `--surface-container-high` #2A2A2A, `--surface-container-highest` #353534, `--surface-container-lowest` #0E0E0E, `--surface-bright` #3A3939
- Primary: `--primary` #FFB87B, `--primary-container` #FF8F00, `--on-primary` #131313
- Text: `--on-surface` #E6E1DD, `--on-surface-variant` #DCC1AE
- Outline: `--outline` #A48C7A, `--outline-variant` #564334
- Tertiary: `--tertiary` #87CFFF
- Status: `--error` #EF5350, `--warning` #FFB300, `--success` #66BB6A

### T2. Import fonts in `tailwind.css`

Import Space Grotesk (weights 500, 700) and Inter (weights 400, 500, 600). Define `--font-display: 'Space Grotesk', sans-serif` and `--font-body: 'Inter', sans-serif`.

### T3. `app.rs` — update CSS variable block

Replace the existing `--foreground` / chart vars with the full token set from T1. Keep chart-specific vars. Update the inline `<style>` block.

### T4. `shell.rs` — restyle container

- `bg-gray-900` → `bg-[var(--surface)]`
- `text-gray-100` → `text-[var(--on-surface)]`
- `p-4` on main → `p-0` (panes fill edge-to-edge, padding managed by individual panes)

### T5. `sidebar.rs` — full restyle

- `bg-gray-800` → `bg-[var(--surface-container-low)]`
- Remove any border between sidebar and content (No-Line Rule)
- Logo `text-blue-400` → `text-[var(--primary)]`, font-family `font-[var(--font-display)]`
- Nav items `text-gray-300` → `text-[var(--outline)]`
- Nav hover `hover:bg-gray-700` → `hover:bg-[var(--surface-container-high)]`
- Active nav item: `bg-[var(--surface-container-high)]` pill + 2px left `bg-[var(--primary)]` indicator bar
- Active nav text: `text-[var(--primary)]`
- Section labels (AGENTS, TERMINALS, etc.): `uppercase tracking-[0.05em] text-xs font-[var(--font-body)]`
- Collapse button `text-gray-500` → `text-[var(--outline)]`

### T6. `tab_bar.rs` — full restyle

- Tab container `bg-gray-950` → `bg-[var(--surface-container)]`
- Active tab `bg-gray-800 border-b-2 border-blue-400` → `bg-[var(--surface-container-high)] border-b-2 border-[var(--primary)]`
- Inactive tab `bg-gray-950 hover:bg-gray-800` → `bg-transparent hover:bg-[var(--primary)]/10 hover:backdrop-blur-sm`
- New tab button `text-gray-400 hover:text-white hover:bg-gray-800` → `text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)]`
- Dropdown menu: remove `border border-gray-600 shadow-lg`, add `bg-[var(--surface-container-high)]/80 backdrop-blur-[12px] shadow-ambient`
- Dropdown items `hover:bg-gray-700` → `hover:bg-[var(--surface-bright)]`
- Dropdown dividers `border-t border-gray-600` → `border-t border-[var(--outline-variant)]/15`
- Input field `bg-gray-900 border border-gray-600` → `bg-[var(--surface-container-lowest)]` no border, focus: `bg-[var(--surface-container-low)] border-b border-[var(--primary)]`
- Submit button `bg-blue-600 hover:bg-blue-500` → `bg-gradient-to-r from-[var(--primary)] to-[var(--primary-container)] text-[var(--on-primary)]`
- Notification dots: map to design tokens (`--error`, `--warning`, `--success`, `--primary` for info)

### T7. `toolbar.rs` — full restyle

- Container `bg-gray-800 border-b border-gray-700` → `bg-[var(--surface-container)]`, remove border entirely
- Control buttons `bg-gray-700 rounded hover:bg-gray-600` → `bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)]`
- Conversion dropdown: same glassmorphism as T6 dropdown
- Dropdown items `hover:bg-gray-700` → `hover:bg-[var(--surface-bright)]`
- Pane title text: derive from command/working_dir, display as tracked uppercase `text-xs uppercase tracking-[0.05em]`

### T8. `view.rs` — terminal container restyle

- TerminalView `bg-gray-950` → `bg-[var(--surface-container-lowest)]`
- Add `p-[1.1rem]` padding to terminal container div for breathing room

### T9. `terminal.ts` — no change needed

The xterm theme (#0E0E0E bg, #DCC1AE fg, #FFB87B cursor) already matches the design system.

### T10. `terminals.rs` — full restyle

- Tab content container `bg-gray-900` → `bg-[var(--surface)]`
- Focused pane border `border-blue-400` → `border-[var(--primary)]`
- Unfocused pane border `border-gray-700` → `border-[var(--outline-variant)]/15`
- Divider base `bg-gray-700` → `bg-[var(--surface-bright)]`
- Divider hover `hover:bg-blue-500/50` → `hover:bg-[var(--primary)]/50`
- Empty state text `text-gray-400` → `text-[var(--outline)]`
- Empty state button: amber gradient per T6

### T11. Create status bar component

New file `layout/status_bar.rs`. Rendered at bottom of Shell. Contents:
- Left: app name + version in tracked uppercase (`text-xs uppercase tracking-[0.05em]`)
- Right: cursor position, encoding, formatter status
- Background: `bg-[var(--surface-container-low)]`
- Text: `text-[var(--outline)]`
- Height: `h-6`
- Font: `font-[var(--font-body)]`

### T12. Add "RUN LX" button to terminals page

In `terminals.rs` tab bar area, right side. Amber gradient button with play icon. `bg-gradient-to-r from-[var(--primary)] to-[var(--primary-container)] text-[var(--on-primary)] rounded-md px-4 py-1.5 text-sm font-medium`.

### T13. Add layout manager to sidebar

Below the nav items in `sidebar.rs`, add a "LAYOUT MANAGER" section label (tracked uppercase). Below it, render saved layout entries as selectable pills. Active layout: `bg-[var(--surface-container-high)]` with `text-[var(--primary)]`. Inactive: `text-[var(--outline)]`.

### T14. Toolbar pane titles

In `toolbar.rs`, derive a meaningful title from the pane's working directory or command (e.g., last path segment, or command name). Display as tracked uppercase label.

### T15. Error and loading state colors across all files

`app.rs` uses `text-red-500` and `text-gray-500`. `shell.rs` uses `text-red-400` and `text-gray-500`. `terminals.rs` uses `text-gray-400`. All cool-tone. Replace error text with `text-[var(--error)]`, loading/empty text with `text-[var(--outline)]`.

### T16. Other view container backgrounds

`view.rs` BrowserView, EditorView, AgentView, CanvasView, VoiceView all use bare `w-full h-full` with no background. Add `bg-[var(--surface-container-lowest)]` to Editor and Voice (recessed content). Add `bg-[var(--surface-container)]` to Browser, Agent, Canvas.

### T17. Collapsed sidebar state

`sidebar.rs` collapsed state uses `w-12 px-1 py-4` with same `bg-gray-800`. Needs `bg-[var(--surface-container-low)]`. Icon-only items in collapsed state should use `text-[var(--outline)]` inactive, `text-[var(--primary)]` active, centered with `justify-center`.

### T18. Tab close button

`tab_bar.rs` close button uses `ml-1 text-xs opacity-50 hover:opacity-100`. Restyle to `text-[var(--outline)] hover:text-[var(--on-surface)]` instead of opacity toggling.

### T19. Custom scrollbar styling

Add scrollbar CSS in `tailwind.css` targeting `::-webkit-scrollbar`. Track: `--surface-container-low`. Thumb: `--surface-container-highest`. Thumb hover: `--surface-bright`. Width: 6px. No border-radius (industrial feel).

### T20. Define ambient shadow utility in `tailwind.css`

Create a custom class `.shadow-ambient` implementing `box-shadow: 0 8px 32px rgba(30, 20, 10, 0.06)`. Amber-tinted charcoal shadow for all floating elements. Use across all dropdowns and modals.

### T21. Define ghost border utility in `tailwind.css`

Create `.border-ghost` implementing `border: 1px solid rgba(86, 67, 52, 0.15)` (outline-variant at 15%). Use wherever a structural boundary is strictly required for accessibility.

### T22. Chart container styling

`view.rs` ChartView uses `w-full h-full min-h-32` with no background. Add `bg-[var(--surface-container)]`.

### T23. Glassmorphism on terminal tab hover

Per design spec, terminal tabs should use 20% transparent primary wash over blurred background on hover. Inactive tab hover: `hover:bg-[var(--primary)]/10 hover:backdrop-blur-sm`.

### T24. Transition consistency

`sidebar.rs` uses `transition-all duration-200`. `toolbar.rs` uses `transition-opacity`. Standardize all hover transitions to `transition-colors duration-150` for color changes, `transition-all duration-200` for layout changes (sidebar collapse).

### T25. Remove tracing debug lines

`view.rs` still has three `dioxus::logger::tracing::info!` debug lines added during terminal debugging. Remove them.
