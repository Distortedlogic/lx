# WU-14: Light Mode Theme

## Fixes
- Fix 1: Add light-mode CSS custom property values under a `.light` class selector
- Fix 2: Add `.dark` class selector wrapping the existing dark-mode values
- Fix 3: Set default `:root` to dark (keep current behavior)
- Fix 4: Apply the theme class to the shell root element
- Fix 5: Wire `ThemeState.toggle()` to add/remove the `.dark`/`.light` class on the root element
- Fix 6: Add light-mode chart colors
- Fix 7: Add light-mode scrollbar styles
- Fix 8: Add light-mode animation background colors
- Fix 9: Ensure the `body` background follows the theme
- Fix 10: Update `ThemeState::provide()` to apply the initial class on mount
- Fix 11: Update `ThemeState::set()` and `ThemeState::toggle()` to apply class changes via JS eval

## Files Modified
- `crates/lx-desktop/src/tailwind.css` (196 lines)
- `crates/lx-desktop/src/contexts/theme.rs` (43 lines)
- `crates/lx-desktop/src/layout/shell.rs` (271 lines) -- minor: apply theme class to root div

## Preconditions
- `tailwind.css` `:root` block with dark-mode CSS custom properties at lines 38-66
- `tailwind.css` scrollbar styles at lines 81-95 using `var(--surface-container-low)` and `var(--surface-container-highest)`
- `tailwind.css` `activity-row-enter` animation at lines 106-119 using hardcoded `rgba(156, 255, 147, ...)`
- `theme.rs` has `Theme` enum with `Light` and `Dark` variants at lines 3-8
- `theme.rs` has `ThemeState` struct with `provide()`, `toggle()`, `set()` methods at lines 10-43
- `shell.rs` line 46: `let _theme = crate::contexts::theme::ThemeState::provide();`
- `shell.rs` line 110: root div has class string `"relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col"`

## Steps

### Step 1: Restructure CSS custom properties in tailwind.css -- dark mode under `.dark` class
- Open `crates/lx-desktop/src/tailwind.css`
- At lines 38-66, find the `:root { ... }` block containing all custom properties
- Replace `:root {` with `:root, .dark {`
- WU-04 may have added `--chart-1` through `--chart-5` lines before the closing `}` of `:root`. The find pattern matches the `:root {` opening line only, so the replacement works regardless of whether WU-04 has already run.

Find:
```
:root {
```
Replace with:
```
:root, .dark {
```

### Step 2: Add `.light` class block with light-mode values
- After the closing `}` of the `:root, .dark` block, add the `.light` block
- The find pattern matches the `body {` line that follows the `:root` block, so it works regardless of whether WU-04 added extra lines inside `:root`

Find:
```
body {
```
Insert immediately before it:
```css
.light {
  --surface: #ffffff;
  --surface-container-low: #f5f5f5;
  --surface-container: #eeeeee;
  --surface-container-high: #e0e0e0;
  --surface-container-highest: #d6d6d6;
  --surface-container-lowest: #fafafa;
  --surface-bright: #f0f0f0;

  --primary: #1a7d1a;
  --primary-container: #a8f09a;
  --on-primary: #ffffff;

  --on-surface: #1a1a1a;
  --on-surface-variant: #555555;

  --outline: #888888;
  --outline-variant: #cccccc;

  --tertiary: #0077aa;

  --error: #cc3300;
  --warning: #cc8800;
  --success: #1a7d1a;

  --color-chart-axis: #cccccc;
  --color-chart-split: #e0e0e0;
  --color-chart-tooltip: #ffffff;

  --chart-1: #15803d;
  --chart-2: #0077aa;
  --chart-3: #b45309;
  --chart-4: #cc3300;
  --chart-5: #7c3aed;
}

```

### Step 3: Update scrollbar styles for theme awareness
- At lines 81-95, find the scrollbar styles
- No changes needed -- these already use CSS custom properties that will automatically pick up light/dark values

### Step 4: Update activity-row-enter animation for theme awareness
- At lines 106-119, find the `@keyframes activity-row-enter` block
- Replace the hardcoded rgba values with `color-mix()` using `var(--primary)`:

Find:
```css
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
```
Replace with:
```css
@keyframes activity-row-enter {
  0% {
    opacity: 0;
    background-color: color-mix(in srgb, var(--primary) 8%, transparent);
  }
  40% {
    opacity: 1;
    background-color: color-mix(in srgb, var(--primary) 6%, transparent);
  }
  100% {
    opacity: 1;
    background-color: transparent;
  }
}
```

### Step 5: Update body style for theme awareness
- At line 69, find:
```css
body {
  background-color: var(--surface);
```
- No change needed -- `var(--surface)` will resolve correctly under both `.dark` and `.light`

### Step 6: Update ThemeState to apply CSS class via JS eval
- Open `crates/lx-desktop/src/contexts/theme.rs`
- Replace the entire file content with:

Find (entire file, lines 1-43):
```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
  Light,
  #[default]
  Dark,
}

#[derive(Clone, Copy)]
pub struct ThemeState {
  pub theme: Signal<Theme>,
}

impl ThemeState {
  pub fn provide() -> Self {
    let state = Self { theme: Signal::new(Theme::Dark) };
    use_context_provider(|| state);
    state
  }

  pub fn current(&self) -> Theme {
    *self.theme.read()
  }

  pub fn set(&self, theme: Theme) {
    let mut sig = self.theme;
    sig.set(theme);
  }

  pub fn toggle(&self) {
    let mut sig = self.theme;
    let next = match *sig.read() {
      Theme::Dark => Theme::Light,
      Theme::Light => Theme::Dark,
    };
    sig.set(next);
  }

  pub fn is_dark(&self) -> bool {
    *self.theme.read() == Theme::Dark
  }
}
```

Replace with:
```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
  Light,
  #[default]
  Dark,
}

impl Theme {
  pub fn css_class(&self) -> &'static str {
    match self {
      Theme::Dark => "dark",
      Theme::Light => "light",
    }
  }
}

#[derive(Clone, Copy)]
pub struct ThemeState {
  pub theme: Signal<Theme>,
}

impl ThemeState {
  pub fn provide() -> Self {
    let state = Self { theme: Signal::new(Theme::Dark) };
    use_context_provider(|| state);
    apply_theme_class(Theme::Dark);
    state
  }

  pub fn current(&self) -> Theme {
    *self.theme.read()
  }

  pub fn set(&self, theme: Theme) {
    let mut sig = self.theme;
    sig.set(theme);
    apply_theme_class(theme);
  }

  pub fn toggle(&self) {
    let mut sig = self.theme;
    let next = match *sig.read() {
      Theme::Dark => Theme::Light,
      Theme::Light => Theme::Dark,
    };
    sig.set(next);
    apply_theme_class(next);
  }

  pub fn is_dark(&self) -> bool {
    *self.theme.read() == Theme::Dark
  }
}

fn apply_theme_class(theme: Theme) {
  let class = theme.css_class();
  let remove = match theme {
    Theme::Dark => "light",
    Theme::Light => "dark",
  };
  let js = format!(
    "document.documentElement.classList.remove('{remove}'); document.documentElement.classList.add('{class}');"
  );
  spawn(async move {
    let _ = document::eval(&js).await;
  });
}
```

### Step 7: No changes needed in shell.rs
- The root `div` in shell.rs at line 110 uses `bg-[var(--surface)]` and `text-[var(--on-surface)]` which already reference CSS custom properties
- The theme class is applied to `document.documentElement` (the `<html>` tag) which is an ancestor of this div, so the CSS cascade handles it correctly
- No code changes needed in shell.rs

## File Size Check
- `tailwind.css`: was 196 lines, now ~232 lines (added ~36 lines for `.light` block) -- under 300
- `theme.rs`: was 43 lines, now ~68 lines -- under 300
- `shell.rs`: unchanged at 271 lines -- under 300

## Verification
- Run `just diagnose` to confirm compilation
- Launch the desktop app:
  1. Default appearance should be unchanged (dark mode)
  2. If you wire a temporary button to call `ThemeState::toggle()` (or call it from the command palette), the entire UI should switch to light mode:
     - Background becomes white/light gray
     - Text becomes dark
     - Primary color becomes a darker green suitable for light backgrounds
     - All CSS-variable-based colors update throughout the app
  3. Toggle back to dark -- all colors revert to the original dark theme
  4. Scrollbars, toast animations, and dialog overlays all look correct in both modes
  5. Inspect `<html>` element in webview dev tools -- it should have class `dark` by default, switching to `light` on toggle
