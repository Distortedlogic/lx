# Unit 1: UI Primitive Components (Part 1 -- Core Inputs)

## Scope

Port 11 core UI primitive components from Paperclip (React/shadcn) to Dioxus 0.7.3 in `lx-desktop`. These are leaf-level components with no inter-component dependencies (except `avatar` uses no other UI component). Each component becomes a Rust file under `src/components/ui/` with Tailwind class strings ported verbatim from the React originals and `data-slot` attributes preserved.

## Preconditions

- `lx-desktop` crate exists at `/home/entropybender/repos/lx/crates/lx-desktop/`
- `dioxus` is already a dependency with `router` and `fullstack` features
- `src/lib.rs` exists and declares modules
- `src/components/` directory does NOT exist yet (must be created)
- Tailwind CSS is loaded via `src/app.rs` (`TAILWIND_CSS` asset)

## File Inventory

All paths relative to `/home/entropybender/repos/lx/crates/lx-desktop/src/`.

| Action | File |
|--------|------|
| CREATE | `components/mod.rs` |
| CREATE | `components/ui/mod.rs` |
| CREATE | `components/ui/button.rs` |
| CREATE | `components/ui/input.rs` |
| CREATE | `components/ui/textarea.rs` |
| CREATE | `components/ui/label.rs` |
| CREATE | `components/ui/checkbox.rs` |
| CREATE | `components/ui/select.rs` |
| CREATE | `components/ui/badge.rs` |
| CREATE | `components/ui/separator.rs` |
| CREATE | `components/ui/skeleton.rs` |
| CREATE | `components/ui/avatar.rs` |
| MODIFY | `lib.rs` |

## Utility: `cn` function

Every Paperclip component uses `cn()` (a class-merging utility). In Dioxus, there is no runtime class merging library. Implement a minimal `cn` function inside `components/ui/mod.rs` that concatenates non-empty class strings with a space separator.

In `components/ui/mod.rs`:

```rust
pub fn cn(classes: &[&str]) -> String {
    classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
```

This function is used by every component below.

## Step 1: Create directory structure and module files

### 1a. Create `components/mod.rs`

File: `src/components/mod.rs`

This is the canonical `components/mod.rs` for the entire project. All subsequent units that add components will edit this file (not recreate it). The initial content includes `pub mod ui;` and forward-declares all component modules that will be added by later units:

```rust
pub mod ui;
pub mod status_colors;
pub mod status_icon;
pub mod status_badge;
pub mod priority_icon;
pub mod identity;
pub mod empty_state;
pub mod entity_row;
pub mod filter_bar;
pub mod copy_text;
pub mod page_skeleton;
pub mod page_tab_bar;
pub mod markdown_body;
pub mod metric_card;
pub mod toast_viewport;
pub mod inline_editor;
pub mod inline_entity_selector;
pub mod comment_thread;
pub mod command_palette;
pub mod company_pattern_icon;
pub mod company_switcher;
pub mod file_tree;
pub mod onboarding;
```

Note: Modules for components not yet created will cause compilation errors until their units are completed. To avoid this, each `pub mod X;` line should be added only when Unit N that creates `X.rs` is executed. The list above is the final target state. During incremental builds, add each `pub mod` declaration as part of the unit that creates the corresponding file.

### 1b. Create `components/ui/mod.rs`

File: `src/components/ui/mod.rs`

```rust
pub mod avatar;
pub mod badge;
pub mod button;
pub mod checkbox;
pub mod input;
pub mod label;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod textarea;

pub fn cn(classes: &[&str]) -> String {
    classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
```

### 1c. Modify `lib.rs`

Add `pub mod components;` to the existing module declarations. Insert it alphabetically (before `pub mod contexts`).

Current `lib.rs` content:
```rust
pub mod app;
pub mod contexts;
pub mod layout;
pub mod pages;
pub mod panes;
pub mod routes;
pub mod styles;
pub mod terminal;
pub mod voice_backend;
#[cfg(feature = "desktop")]
pub mod webview_permissions;
```

Add `pub mod components;` after `pub mod app;`.

## Step 2: Port `button.rs`

**Source:** `reference/paperclip/ui/src/components/ui/button.tsx`

File: `src/components/ui/button.rs`

### Struct: `ButtonVariant`

```rust
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}
```

### Struct: `ButtonSize`

```rust
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}
```

### Function: `button_variant_class`

Returns the Tailwind class string for a given `(ButtonVariant, ButtonSize)` pair. Use a match on variant, then a match on size, concatenating the base classes with variant and size classes.

Base class (verbatim from Paperclip):
```
"inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-[color,background-color,border-color,box-shadow,opacity] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive"
```

Variant classes (verbatim):
- `Default`: `"bg-primary text-primary-foreground hover:bg-primary/90"`
- `Destructive`: `"bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60"`
- `Outline`: `"border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50"`
- `Secondary`: `"bg-secondary text-secondary-foreground hover:bg-secondary/80"`
- `Ghost`: `"hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50"`
- `Link`: `"text-primary underline-offset-4 hover:underline"`

Size classes (verbatim):
- `Default`: `"h-10 px-4 py-2 has-[>svg]:px-3"`
- `Xs`: `"h-6 gap-1 rounded-md px-2 text-xs has-[>svg]:px-1.5 [&_svg:not([class*='size-'])]:size-3"`
- `Sm`: `"h-9 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5"`
- `Lg`: `"h-10 rounded-md px-6 has-[>svg]:px-4"`
- `Icon`: `"size-10"`
- `IconXs`: `"size-6 rounded-md [&_svg:not([class*='size-'])]:size-3"`
- `IconSm`: `"size-9"`
- `IconLg`: `"size-10"`

### Component: `Button`

```rust
#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] class: String,
    #[props(default)] disabled: bool,
    #[props(default)] r#type: String,
    children: Element,
) -> Element
```

Renders a `<button>` with:
- `data-slot="button"`
- `data-variant` set to lowercase variant name string
- `data-size` set to lowercase size name string
- `class` set to `cn(&[button_variant_class(variant, size), &class])`
- `disabled` attribute forwarded
- `type` attribute forwarded (default `"button"`)
- `{children}` as body

## Step 3: Port `input.rs`

**Source:** `reference/paperclip/ui/src/components/ui/input.tsx`

File: `src/components/ui/input.rs`

### Component: `Input`

```rust
#[component]
pub fn Input(
    #[props(default)] class: String,
    #[props(default = "text".to_string())] r#type: String,
    #[props(default)] placeholder: String,
    #[props(default)] value: String,
    #[props(default)] disabled: bool,
    #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element
```

Renders `<input>` with:
- `data-slot="input"`
- `r#type` as `type` attribute
- `class` set to `cn(&[BASE_INPUT_CLASS, &class])`
- `disabled`, `placeholder`, `value` forwarded
- `oninput` handler forwarded

`BASE_INPUT_CLASS` constant (verbatim):
```
"file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive"
```

## Step 4: Port `textarea.rs`

**Source:** `reference/paperclip/ui/src/components/ui/textarea.tsx`

File: `src/components/ui/textarea.rs`

### Component: `Textarea`

```rust
#[component]
pub fn Textarea(
    #[props(default)] class: String,
    #[props(default)] placeholder: String,
    #[props(default)] value: String,
    #[props(default)] disabled: bool,
    #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element
```

Renders `<textarea>` with:
- `data-slot="textarea"`
- `class` set to `cn(&[BASE_TEXTAREA_CLASS, &class])`
- `disabled`, `placeholder`, `value` forwarded
- `oninput` handler forwarded

`BASE_TEXTAREA_CLASS` constant (verbatim):
```
"border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 md:text-sm"
```

## Step 5: Port `label.rs`

**Source:** `reference/paperclip/ui/src/components/ui/label.tsx`

File: `src/components/ui/label.rs`

### Component: `Label`

```rust
#[component]
pub fn Label(
    #[props(default)] class: String,
    #[props(default)] r#for: String,
    children: Element,
) -> Element
```

Renders `<label>` with:
- `data-slot="label"`
- `r#for` as `for` attribute
- `class` set to `cn(&[BASE_LABEL_CLASS, &class])`
- `{children}` as body

`BASE_LABEL_CLASS` constant (verbatim):
```
"flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50"
```

## Step 6: Port `checkbox.rs`

**Source:** `reference/paperclip/ui/src/components/ui/checkbox.tsx`

File: `src/components/ui/checkbox.rs`

### Component: `Checkbox`

```rust
#[component]
pub fn Checkbox(
    #[props(default)] class: String,
    #[props(default)] checked: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onchange: EventHandler<FormEvent>,
) -> Element
```

Renders a `<button>` (checkbox is a toggle button in Dioxus since there is no Radix):
- `data-slot="checkbox"`
- `role="checkbox"`
- `aria-checked="{checked}"`
- `class` set to `cn(&[BASE_CHECKBOX_CLASS, &class])`
- `disabled` forwarded
- `onclick` handler that invokes `onchange`
- When `checked` is true, renders an inner `<svg>` checkmark icon (a simple polyline check: `<svg viewBox="0 0 24 24" class="size-3.5"><polyline points="20 6 9 17 4 12" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" /></svg>`)

`BASE_CHECKBOX_CLASS` constant (verbatim):
```
"peer border-input dark:bg-input/30 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive size-4 shrink-0 rounded-[4px] border shadow-xs transition-shadow outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50"
```

Also set `data-state` to `"checked"` or `"unchecked"` based on the `checked` prop, and render the inner indicator div with `class="grid place-content-center text-current transition-none"` wrapping the SVG checkmark, only when `checked` is true.

## Step 7: Port `select.rs`

**Source:** `reference/paperclip/ui/src/components/ui/select.tsx`

File: `src/components/ui/select.rs`

Since Dioxus does not have Radix UI primitives, port as a native `<select>` wrapper with styled container.

### Component: `Select`

```rust
#[component]
pub fn Select(
    #[props(default)] class: String,
    #[props(default)] value: String,
    #[props(default)] disabled: bool,
    #[props(default)] onchange: EventHandler<FormEvent>,
    children: Element,
) -> Element
```

Renders a `<div data-slot="select">` wrapping a native `<select>` element with:
- `data-slot="select-trigger"` on the `<select>`
- `class` on the `<select>` set to `cn(&[BASE_SELECT_TRIGGER_CLASS, &class])`
- `disabled`, `value`, `onchange` forwarded
- `{children}` (expected to be `<option>` elements) as body

`BASE_SELECT_TRIGGER_CLASS` constant (verbatim):
```
"border-input data-[placeholder]:text-muted-foreground [&_svg:not([class*='text-'])]:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 dark:hover:bg-input/50 flex w-fit items-center justify-between gap-2 rounded-md border bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 h-9"
```

### Component: `SelectItem`

```rust
#[component]
pub fn SelectItem(
    value: String,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element
```

Renders a native `<option>` with `value` and `disabled` forwarded, `{children}` as body.

## Step 8: Port `badge.rs`

**Source:** `reference/paperclip/ui/src/components/ui/badge.tsx`

File: `src/components/ui/badge.rs`

### Struct: `BadgeVariant`

```rust
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}
```

### Function: `badge_variant_class`

Returns concatenation of base + variant class.

Base class (verbatim):
```
"inline-flex items-center justify-center rounded-full border border-transparent px-2 py-0.5 text-xs font-medium w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 gap-1 [&>svg]:pointer-events-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive transition-[color,box-shadow] overflow-hidden"
```

Variant classes (verbatim):
- `Default`: `"bg-primary text-primary-foreground [a&]:hover:bg-primary/90"`
- `Secondary`: `"bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90"`
- `Destructive`: `"bg-destructive text-white [a&]:hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60"`
- `Outline`: `"border-border text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground"`
- `Ghost`: `"[a&]:hover:bg-accent [a&]:hover:text-accent-foreground"`
- `Link`: `"text-primary underline-offset-4 [a&]:hover:underline"`

### Component: `Badge`

```rust
#[component]
pub fn Badge(
    #[props(default)] variant: BadgeVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<span>` with:
- `data-slot="badge"`
- `data-variant` set to lowercase variant name
- `class` set to `cn(&[badge_variant_class(variant), &class])`
- `{children}` as body

## Step 9: Port `separator.rs`

**Source:** `reference/paperclip/ui/src/components/ui/separator.tsx`

File: `src/components/ui/separator.rs`

### Struct: `Orientation`

```rust
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
```

### Component: `Separator`

```rust
#[component]
pub fn Separator(
    #[props(default)] orientation: Orientation,
    #[props(default = true)] decorative: bool,
    #[props(default)] class: String,
) -> Element
```

Renders a `<div>` with:
- `data-slot="separator"`
- `role` set to `"none"` if `decorative`, else `"separator"`
- `aria-orientation` set to `"horizontal"` or `"vertical"`
- `data-orientation` set to `"horizontal"` or `"vertical"`
- `class` set to `cn(&[BASE_SEPARATOR_CLASS, orientation_class, &class])`

`BASE_SEPARATOR_CLASS`: `"bg-border shrink-0"`

Orientation-specific classes:
- `Horizontal`: `"h-px w-full"`
- `Vertical`: `"h-full w-px"`

## Step 10: Port `skeleton.rs`

**Source:** `reference/paperclip/ui/src/components/ui/skeleton.tsx`

File: `src/components/ui/skeleton.rs`

### Component: `Skeleton`

```rust
#[component]
pub fn Skeleton(
    #[props(default)] class: String,
) -> Element
```

Renders a `<div>` with:
- `data-slot="skeleton"`
- `class` set to `cn(&["bg-accent/75 rounded-md animate-pulse", &class])`

Use `animate-pulse` class for the shimmer animation. The class string is: `"bg-accent/75 rounded-md animate-pulse"`.

## Step 11: Port `avatar.rs`

**Source:** `reference/paperclip/ui/src/components/ui/avatar.tsx`

File: `src/components/ui/avatar.rs`

### Struct: `AvatarSize`

```rust
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
}
```

### Component: `Avatar`

```rust
#[component]
pub fn Avatar(
    #[props(default)] size: AvatarSize,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<span>` with:
- `data-slot="avatar"`
- `data-size` set to lowercase size name (`"default"`, `"xs"`, `"sm"`, `"lg"`)
- `class` set to `cn(&[BASE_AVATAR_CLASS, &class])`
- `{children}` as body

`BASE_AVATAR_CLASS` (verbatim):
```
"group/avatar relative flex size-8 shrink-0 overflow-hidden rounded-full select-none data-[size=lg]:size-10 data-[size=sm]:size-6 data-[size=xs]:size-5"
```

### Component: `AvatarImage`

```rust
#[component]
pub fn AvatarImage(
    src: String,
    #[props(default)] alt: String,
    #[props(default)] class: String,
) -> Element
```

Renders `<img>` with:
- `data-slot="avatar-image"`
- `src`, `alt` forwarded
- `class` set to `cn(&["aspect-square size-full", &class])`

### Component: `AvatarFallback`

```rust
#[component]
pub fn AvatarFallback(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<span>` with:
- `data-slot="avatar-fallback"`
- `class` set to `cn(&[AVATAR_FALLBACK_CLASS, &class])`
- `{children}` as body

`AVATAR_FALLBACK_CLASS` (verbatim):
```
"bg-muted text-muted-foreground flex size-full items-center justify-center rounded-full text-sm group-data-[size=sm]/avatar:text-xs group-data-[size=xs]/avatar:text-[10px]"
```

### Component: `AvatarBadge`

```rust
#[component]
pub fn AvatarBadge(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<span>` with:
- `data-slot="avatar-badge"`
- `class` set to `cn(&[AVATAR_BADGE_CLASS, &class])`
- `{children}` as body

`AVATAR_BADGE_CLASS` (verbatim -- combine the multi-line classes):
```
"bg-primary text-primary-foreground ring-background absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full ring-2 select-none group-data-[size=sm]/avatar:size-2 group-data-[size=sm]/avatar:[&>svg]:hidden group-data-[size=default]/avatar:size-2.5 group-data-[size=default]/avatar:[&>svg]:size-2 group-data-[size=lg]/avatar:size-3 group-data-[size=lg]/avatar:[&>svg]:size-2"
```

### Component: `AvatarGroup`

```rust
#[component]
pub fn AvatarGroup(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div>` with:
- `data-slot="avatar-group"`
- `class` set to `cn(&["*:data-[slot=avatar]:ring-background group/avatar-group flex -space-x-2 *:data-[slot=avatar]:ring-2", &class])`
- `{children}` as body

## Definition of Done

1. `just diagnose` passes with zero errors and zero warnings for the `lx-desktop` crate
2. All 12 files listed in the File Inventory exist at the specified paths
3. `src/lib.rs` contains `pub mod components;`
4. `src/components/mod.rs` contains `pub mod ui;`
5. `src/components/ui/mod.rs` re-exports all 10 submodules and the `cn` function
6. Each component file contains the exact `data-slot` attributes specified above
7. Each component file contains the exact Tailwind class strings specified above (verbatim, not paraphrased)
8. No file exceeds 300 lines
9. No `#[allow(...)]` attributes are used
10. No doc comments or code comments are present
