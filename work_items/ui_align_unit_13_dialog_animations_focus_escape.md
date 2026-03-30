# Unit 13: Dialog Animations + Focus Trap + Escape

## Goal

Add backdrop fade-in, content zoom-in CSS animations, Escape-to-close via onkeydown on the dialog itself, and a focus trap that cycles Tab between focusable elements within the dialog.

## Preconditions

- No other units need to be complete first
- The `DialogContent` component in `crates/lx-desktop/src/components/ui/dialog.rs` currently works: it shows/hides based on `open` signal, has a close button, and the backdrop `onclick` closes the dialog

## Files to Modify

- `crates/lx-desktop/src/tailwind.css`
- `crates/lx-desktop/src/components/ui/dialog.rs`

## Steps

### Step 1: Add CSS @keyframes to `tailwind.css`

Append after the last animation block in the file (after `.animate-activity-enter` or after the toast animations if Unit 12 was completed first):

```css
@keyframes dialog-overlay-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.animate-dialog-overlay-in {
  animation: dialog-overlay-in 200ms ease-out both;
}

@keyframes dialog-content-in {
  from {
    opacity: 0;
    transform: translate(-50%, -50%) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }
}

.animate-dialog-content-in {
  animation: dialog-content-in 200ms ease-out both;
}
```

### Step 2: Replace `animate-fade-in` on the dialog overlay

In `dialog.rs`, the overlay div (line 21) currently has `class: "fixed inset-0 z-50 bg-black/50 animate-fade-in"`.

Replace `animate-fade-in` with `animate-dialog-overlay-in`:

Old:
```
class: "fixed inset-0 z-50 bg-black/50 animate-fade-in",
```

New:
```
class: "fixed inset-0 z-50 bg-black/50 animate-dialog-overlay-in",
```

### Step 3: Add animation class to the dialog content div

In `dialog.rs`, the content div (line 28-33) builds its class via `cn()`. Add `animate-dialog-content-in` to the base class string.

Old (first argument to `cn`):
```
"bg-background fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg sm:max-w-lg",
```

New:
```
"bg-background fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] gap-4 rounded-lg border p-6 shadow-lg sm:max-w-lg animate-dialog-content-in",
```

Note: Remove `translate-x-[-50%] translate-y-[-50%]` from the Tailwind classes because the `@keyframes dialog-content-in` animation handles the centering via `transform: translate(-50%, -50%) scale(1)`. Having both the Tailwind translate utilities AND the animation keyframe transform will conflict (CSS `transform` is a single property -- the animation's `to` state will override the Tailwind utilities, but the Tailwind utilities would override the animation during static state). The animation's `to` keyframe already includes the centering transform, and once the animation completes with `both` fill mode, it stays at the final state.

### Step 4: Add Escape key handler to `DialogContent`

Add an `onkeydown` handler directly on the dialog content div. This makes the dialog self-contained for Escape handling rather than relying on the global keyboard shortcuts hook.

In `DialogContent` (line 13 onward), add the `onkeydown` to the content div:

```rust
#[component]
pub fn DialogContent(
  open: Signal<bool>,
  #[props(default)] class: String,
  #[props(default = true)] show_close_button: bool,
  children: Element,
) -> Element {
  if !open() {
    return rsx! {};
  }
  let mut open = open;
  rsx! {
    div {
      "data-slot": "dialog-overlay",
      class: "fixed inset-0 z-50 bg-black/50 animate-dialog-overlay-in",
      onclick: move |_| open.set(false),
    }
    div {
      "data-slot": "dialog-content",
      role: "dialog",
      "aria-modal": "true",
      tabindex: "0",
      class: cn(
          &[
              "bg-background fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] gap-4 rounded-lg border p-6 shadow-lg sm:max-w-lg animate-dialog-content-in outline-none",
              &class,
          ],
      ),
      onmounted: move |evt| {
          let el = evt.data();
          spawn(async move {
              let _ = el.set_focus(true).await;
          });
      },
      onkeydown: move |evt: KeyboardEvent| {
          if evt.key() == Key::Escape {
              evt.stop_propagation();
              open.set(false);
          }
      },
      // Preserve the existing close button code (dialog.rs lines 34-59) unchanged.
      {children}
    }
  }
}
```

Key points:
- `tabindex: "0"` makes the div focusable
- `onmounted` auto-focuses the dialog content when it appears so it can receive keyboard events
- `onkeydown` catches Escape and calls `open.set(false)` with `stop_propagation()` so it does not bubble to the global handler
- Add `outline-none` to the `cn` base classes so the focus ring on the dialog container itself is invisible

### Step 5: Add focus trap via JS interop

Add a `onkeydown` handler extension that traps Tab within the dialog. Extend the existing `onkeydown` handler in the content div to also handle Tab:

```rust
onkeydown: move |evt: KeyboardEvent| {
    if evt.key() == Key::Escape {
        evt.stop_propagation();
        open.set(false);
        return;
    }
    if evt.key() == Key::Tab {
        evt.prevent_default();
        let shift = evt.modifiers().shift();
        spawn(async move {
            let direction = if shift { "backward" } else { "forward" };
            let js = format!(
                r#"(function() {{
                    var dialog = document.querySelector('[data-slot="dialog-content"]');
                    if (!dialog) return;
                    var focusable = dialog.querySelectorAll(
                        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
                    );
                    if (focusable.length === 0) return;
                    var arr = Array.from(focusable);
                    var idx = arr.indexOf(document.activeElement);
                    if ('{direction}' === 'forward') {{
                        var next = (idx + 1) % arr.length;
                        arr[next].focus();
                    }} else {{
                        var prev = (idx - 1 + arr.length) % arr.length;
                        arr[prev].focus();
                    }}
                }})()"#
            );
            let _ = document::eval(&js).await;
        });
    }
},
```

This JS interop approach:
- Queries all focusable elements within `[data-slot="dialog-content"]`
- Finds the currently focused element's index
- Moves focus forward (Tab) or backward (Shift+Tab), wrapping around
- `prevent_default()` stops the browser's native tab behavior

### Step 6: Verify the full `DialogContent` component

After all changes, the `DialogContent` function should contain:
1. The early return for `!open()`
2. The overlay div with `animate-dialog-overlay-in`
3. The content div with `tabindex: "0"`, `animate-dialog-content-in`, `outline-none`, `onmounted` for auto-focus, `onkeydown` handling both Escape and Tab
4. The existing close button (unchanged)
5. `{children}` (unchanged)

Ensure the file stays under 300 lines. The other dialog sub-components (`DialogHeader`, `DialogFooter`, `DialogTitle`, `DialogDescription`) remain unchanged.

## Verification

1. Run `just diagnose` -- must compile with zero warnings
2. Visual checks in the running app:
   - Open any dialog (new issue, new project, new agent, onboarding) -- the backdrop should fade in over 200ms and the content should zoom-in/fade-in from 95% scale
   - Press Escape while a dialog is open -- it should close
   - Press Tab repeatedly while a dialog is open -- focus should cycle through focusable elements (buttons, inputs, textareas) within the dialog and never escape to the background
   - Press Shift+Tab -- focus should cycle backward
   - Click the backdrop -- should still close the dialog (existing behavior preserved)
   - Click the X button -- should still close the dialog (existing behavior preserved)
