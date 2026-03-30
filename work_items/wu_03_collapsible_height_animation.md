# WU-03: Collapsible height animation

## Fixes
- Fix 8: `CollapsibleContent` currently uses conditional rendering (`if !open() { return rsx! {} }`), which means it unmounts/remounts content on toggle. Replace with always-mounted content that animates height via CSS grid trick.

## Files Modified
- `crates/lx-desktop/src/components/ui/collapsible.rs` (41 lines)
- `crates/lx-desktop/src/tailwind.css` (196 lines)

## Preconditions
- `CollapsibleContent` at line 33 of `collapsible.rs` returns an empty `rsx!` when closed (line 35-36).
- `tailwind.css` already has keyframe animations (lines 97-196) — the new animation CSS goes in the same file.
- The `cn` utility is imported at line 3 via `use super::cn;`.

## Steps

### Step 1: Replace CollapsibleContent with always-mounted CSS animation approach
- Open `crates/lx-desktop/src/components/ui/collapsible.rs`
- At lines 33-41, find:
```rust
#[component]
pub fn CollapsibleContent(open: Signal<bool>, #[props(default)] class: String, children: Element) -> Element {
  if !open() {
    return rsx! {};
  }
  rsx! {
    div { "data-slot": "collapsible-content", class: cn(&[&class]), {children} }
  }
}
```
- Replace with:
```rust
#[component]
pub fn CollapsibleContent(open: Signal<bool>, #[props(default)] class: String, children: Element) -> Element {
  let data_state = if open() { "open" } else { "closed" };
  let anim_class = if open() { "collapsible-open" } else { "collapsible-closed" };
  rsx! {
    div {
      "data-slot": "collapsible-content",
      "data-state": data_state,
      class: "grid {anim_class} {class}",
      div { class: "overflow-hidden",
        {children}
      }
    }
  }
}
```
- Why: The CSS grid approach uses `grid-template-rows: 0fr` (closed) and `grid-template-rows: 1fr` (open) with a CSS transition. The inner `div` with `overflow-hidden` clips the content. The content stays mounted in the DOM at all times, so state is preserved across toggles.

### Step 2: Add collapsible animation CSS
- Open `crates/lx-desktop/src/tailwind.css`
- At the end of the file (after line 196), add:
```css

.collapsible-open {
  grid-template-rows: 1fr;
  transition: grid-template-rows 200ms ease-out;
}

.collapsible-closed {
  grid-template-rows: 0fr;
  transition: grid-template-rows 200ms ease-out;
}
```
- Why: CSS grid row animation is the modern way to animate height from 0 to auto. The `1fr` / `0fr` trick with `overflow-hidden` on the child gives a smooth height transition without needing to know the content height.

## File Size Check
- `collapsible.rs`: was 41 lines, now ~47 lines (under 300)
- `tailwind.css`: was 196 lines, now ~206 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compilation errors.
- Find any usage of `CollapsibleContent` in the app (search for `CollapsibleContent` in the codebase). Toggle it open/closed and confirm:
  1. The content slides open/closed smoothly over ~200ms.
  2. Content inside the collapsible retains its state (e.g., form inputs keep their values) when toggled closed and reopened.
  3. No layout jump — the content does not suddenly appear/disappear.
