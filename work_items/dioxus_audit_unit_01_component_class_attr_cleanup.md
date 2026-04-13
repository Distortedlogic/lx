# Dioxus Audit Unit 01: Shared Component Class Attribute Cleanup

## Goal

Remove the verified Dioxus audit violations in reusable desktop components where static utility classes and dynamic class fragments are mixed inside the same `class` string. Split those mixed attributes into explicit static and dynamic `class` attributes without changing rendered structure or styling.

## Preconditions

- No earlier Dioxus audit unit is required.
- This unit is intentionally limited to reusable files under `crates/lx-desktop/src/components/`; page-level mixed-class violations remain out of scope for a later unit.

## Verified Findings

- The Dioxus audit rule in `rules/dioxus-audit.md` forbids RSX `class` attributes that combine static classes with interpolated values in one string; the preferred form is one static `class` attribute plus separate dynamic `class` attributes.
- The following shared component files currently violate that rule with patterns such as `"static classes {dynamic}"`, `"static{dynamic}"`, or formatted class strings that join static and dynamic tokens into one value:
  - `crates/lx-desktop/src/components/company_pattern_icon.rs`
  - `crates/lx-desktop/src/components/company_switcher.rs`
  - `crates/lx-desktop/src/components/copy_text.rs`
  - `crates/lx-desktop/src/components/file_tree.rs`
  - `crates/lx-desktop/src/components/identity.rs`
  - `crates/lx-desktop/src/components/inline_editor.rs`
  - `crates/lx-desktop/src/components/inline_entity_selector.rs`
  - `crates/lx-desktop/src/components/markdown_body.rs`
  - `crates/lx-desktop/src/components/markdown_editor.rs`
  - `crates/lx-desktop/src/components/mention_popup.rs`
  - `crates/lx-desktop/src/components/metric_card.rs`
  - `crates/lx-desktop/src/components/page_skeleton.rs`
  - `crates/lx-desktop/src/components/page_tab_bar.rs`
  - `crates/lx-desktop/src/components/priority_icon.rs`
  - `crates/lx-desktop/src/components/scroll_to_bottom.rs`
  - `crates/lx-desktop/src/components/toast_viewport.rs`
  - `crates/lx-desktop/src/components/ui/collapsible.rs`
  - `crates/lx-desktop/src/components/ui/dialog.rs`
- These components are reused across multiple desktop pages, so fixing the component layer removes repeated rule violations without changing page-level control flow or data handling.
- Some violations currently rely on string adjacency like `hover:bg...[...] {bg}` or `min-h-9{sel_class}`. Those must be rewritten so empty dynamic values do not leave malformed merged class tokens or require embedded spacing tricks.

## Files to Modify

- `crates/lx-desktop/src/components/company_pattern_icon.rs`
- `crates/lx-desktop/src/components/company_switcher.rs`
- `crates/lx-desktop/src/components/copy_text.rs`
- `crates/lx-desktop/src/components/file_tree.rs`
- `crates/lx-desktop/src/components/identity.rs`
- `crates/lx-desktop/src/components/inline_editor.rs`
- `crates/lx-desktop/src/components/inline_entity_selector.rs`
- `crates/lx-desktop/src/components/markdown_body.rs`
- `crates/lx-desktop/src/components/markdown_editor.rs`
- `crates/lx-desktop/src/components/mention_popup.rs`
- `crates/lx-desktop/src/components/metric_card.rs`
- `crates/lx-desktop/src/components/page_skeleton.rs`
- `crates/lx-desktop/src/components/page_tab_bar.rs`
- `crates/lx-desktop/src/components/priority_icon.rs`
- `crates/lx-desktop/src/components/scroll_to_bottom.rs`
- `crates/lx-desktop/src/components/toast_viewport.rs`
- `crates/lx-desktop/src/components/ui/collapsible.rs`
- `crates/lx-desktop/src/components/ui/dialog.rs`

## Steps

### Step 1: Split every mixed `class` string in the shared component layer

In each file listed above, replace every `class` value that combines static classes and dynamic fragments in one string with separate `class` attributes:

- Keep the full static utility class list in one literal `class`.
- Move each dynamic fragment into its own `class: "{...}"` or `class: dynamic_value` attribute.
- Where a file currently uses a formatted class string or concatenated suffix for the same purpose, separate the static and dynamic pieces instead of rebuilding the same mixed string through `format!`.

Apply that pattern to the exact dynamic fragments already present in these files, including:

- optional external component class props such as `extra_class`, `extra`, and `class`
- state-driven styling such as `drag_class`, `bg`, `hover_class`, `opacity_class`, `overlay_anim`, `anim_class`, `color`, `dc`, and `sel_class`
- helper-returned classes such as `status_dot_color(...)`

Do not change any non-class attributes, event handlers, or DOM structure unless a tiny RSX reshuffle is required to express separate `class` attributes.

### Step 2: Preserve existing spacing and empty-value behavior

When splitting attributes, do not keep leading or trailing spaces inside the dynamic class variables just to make concatenation work. Rewrite the RSX so each dynamic class fragment stands alone, which keeps empty strings safe and avoids malformed merged tokens such as `min-h-9bg-...`.

For `crates/lx-desktop/src/components/toast_viewport.rs` and `crates/lx-desktop/src/components/ui/dialog.rs`, make sure the animation class still toggles exactly as before after separating the static classes from the dynamic class token.

### Step 3: Keep the scope component-only

Do not touch page files under `crates/lx-desktop/src/pages/` in this unit, even though the audit rule still finds more mixed-class violations there. This unit is only for reusable shared components.

## Verification

1. Run `just fmt`.
2. Run `cargo check -p lx-desktop`.
3. Run `rg -n -P 'class:\\s*\"(?:[^\"{][^\"]*\\{[^}]+\\}[^\"]*|\\{[^}]+\\}[^\"\\s][^\"]*)\"' crates/lx-desktop/src/components --type rust`.
4. Confirm the grep in step 3 returns no matches for the files listed in this work item.
