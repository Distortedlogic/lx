# Unit 2: UI Primitive Components (Part 2 -- Overlays & Containers)

## Scope

Port 11 overlay and container UI components from Paperclip (React/shadcn) to Dioxus 0.7.3 in `lx-desktop`. These components are more structurally complex than Unit 1: they include modal overlays, tabbed containers, tooltip/popover positioning, dropdown menus, collapsible sections, command palettes, scroll areas, and breadcrumb navigation. Since Dioxus lacks Radix UI primitives, each component is implemented with native HTML elements, ARIA attributes, and Dioxus signals for open/close state management.

## Preconditions

- Unit 1 is complete: `src/components/ui/mod.rs` exists with the `cn` function
- `src/components/ui/button.rs` exists (dialog uses the Button component)
- The `dioxus` crate is available with `prelude::*`

## File Inventory

All paths relative to `/home/entropybender/repos/lx/crates/lx-desktop/src/`.

| Action | File |
|--------|------|
| CREATE | `components/ui/dialog.rs` |
| CREATE | `components/ui/sheet.rs` |
| CREATE | `components/ui/card.rs` |
| CREATE | `components/ui/tabs.rs` |
| CREATE | `components/ui/tooltip.rs` |
| CREATE | `components/ui/popover.rs` |
| CREATE | `components/ui/dropdown_menu.rs` |
| CREATE | `components/ui/collapsible.rs` |
| CREATE | `components/ui/command.rs` |
| CREATE | `components/ui/scroll_area.rs` |
| CREATE | `components/ui/breadcrumb.rs` |
| MODIFY | `components/ui/mod.rs` |

## Step 1: Update `components/ui/mod.rs`

Add the 11 new module declarations to the existing `mod.rs`. The full list after modification:

```rust
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod collapsible;
pub mod command;
pub mod dialog;
pub mod dropdown_menu;
pub mod input;
pub mod label;
pub mod popover;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod sheet;
pub mod skeleton;
pub mod tabs;
pub mod textarea;
pub mod tooltip;

pub fn cn(classes: &[&str]) -> String {
    classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
```

## Step 2: Port `dialog.rs`

**Source:** `reference/paperclip/ui/src/components/ui/dialog.tsx`

File: `src/components/ui/dialog.rs`

Since Dioxus has no Radix Dialog primitive, implement as a controlled overlay using a `Signal<bool>` for open state.

### Component: `Dialog`

```rust
#[component]
pub fn Dialog(
    open: Signal<bool>,
    children: Element,
) -> Element
```

Renders nothing visible itself. Wraps `{children}` in a `<div data-slot="dialog">`. The `open` signal is read by `DialogContent` to decide visibility.

### Component: `DialogContent`

```rust
#[component]
pub fn DialogContent(
    open: Signal<bool>,
    #[props(default)] class: String,
    #[props(default = true)] show_close_button: bool,
    children: Element,
) -> Element
```

When `*open.read()` is `false`, render `None`. When `true`, render:

1. A backdrop overlay `<div data-slot="dialog-overlay">` with class (verbatim):
   ```
   "fixed inset-0 z-50 bg-black/50"
   ```
   With `onclick` that sets `open` to `false`.

2. A content container `<div data-slot="dialog-content" role="dialog" aria-modal="true">` with class (verbatim):
   ```
   "bg-background fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg sm:max-w-lg"
   ```
   merged with `&class`.

3. If `show_close_button` is `true`, render inside the content div a close button `<button data-slot="dialog-close">` with class (verbatim):
   ```
   "ring-offset-background focus:ring-ring absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
   ```
   Containing an X icon SVG (`<svg viewBox="0 0 24 24" class="size-4"><line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>`) and a `<span class="sr-only">"Close"</span>`. The button's `onclick` sets `open` to `false`.

4. `{children}` rendered inside the content div.

### Component: `DialogHeader`

```rust
#[component]
pub fn DialogHeader(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="dialog-header">` with class (verbatim):
```
"flex flex-col gap-2 text-center sm:text-left"
```
merged with `&class`. Contains `{children}`.

### Component: `DialogFooter`

```rust
#[component]
pub fn DialogFooter(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="dialog-footer">` with class (verbatim):
```
"flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"
```
merged with `&class`. Contains `{children}`.

### Component: `DialogTitle`

```rust
#[component]
pub fn DialogTitle(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<h2 data-slot="dialog-title">` with class (verbatim):
```
"text-lg leading-none font-semibold"
```
merged with `&class`. Contains `{children}`.

### Component: `DialogDescription`

```rust
#[component]
pub fn DialogDescription(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<p data-slot="dialog-description">` with class (verbatim):
```
"text-muted-foreground text-sm"
```
merged with `&class`. Contains `{children}`.

**Note:** This file will be close to 150 lines. Keep it under 300.

## Step 3: Port `sheet.rs`

**Source:** `reference/paperclip/ui/src/components/ui/sheet.tsx`

File: `src/components/ui/sheet.rs`

### Struct: `SheetSide`

```rust
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SheetSide {
    Top,
    #[default]
    Right,
    Bottom,
    Left,
}
```

### Component: `SheetContent`

```rust
#[component]
pub fn SheetContent(
    open: Signal<bool>,
    #[props(default)] side: SheetSide,
    #[props(default)] class: String,
    #[props(default = true)] show_close_button: bool,
    children: Element,
) -> Element
```

When `*open.read()` is `false`, render `None`. When `true`, render:

1. Overlay `<div data-slot="sheet-overlay">` with class (verbatim):
   ```
   "fixed inset-0 z-50 bg-black/50"
   ```
   `onclick` sets `open` to `false`.

2. Content `<div data-slot="sheet-content">` with base class (verbatim):
   ```
   "bg-background fixed z-50 flex flex-col gap-4 shadow-lg"
   ```
   Plus side-specific classes:
   - `Right`: `"inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm"`
   - `Left`: `"inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm"`
   - `Top`: `"inset-x-0 top-0 h-auto border-b"`
   - `Bottom`: `"inset-x-0 bottom-0 h-auto border-t"`

3. If `show_close_button`, render close button with X icon (same SVG pattern as dialog) with class (verbatim):
   ```
   "ring-offset-background focus:ring-ring absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none"
   ```

### Component: `SheetHeader`

Renders `<div data-slot="sheet-header">` with class: `"flex flex-col gap-1.5 p-4"`.

### Component: `SheetFooter`

Renders `<div data-slot="sheet-footer">` with class: `"mt-auto flex flex-col gap-2 p-4"`.

### Component: `SheetTitle`

Renders `<h2 data-slot="sheet-title">` with class: `"text-foreground font-semibold"`.

### Component: `SheetDescription`

Renders `<p data-slot="sheet-description">` with class: `"text-muted-foreground text-sm"`.

All accept `#[props(default)] class: String` and `children: Element`, merging class with `cn`.

## Step 4: Port `card.rs`

**Source:** `reference/paperclip/ui/src/components/ui/card.tsx`

File: `src/components/ui/card.rs`

Six pure-layout components. All take `#[props(default)] class: String` and `children: Element`.

### Component: `Card`

`<div data-slot="card">`, class (verbatim):
```
"bg-card text-card-foreground flex flex-col gap-6 border py-6 shadow-sm"
```

### Component: `CardHeader`

`<div data-slot="card-header">`, class (verbatim):
```
"@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6"
```

### Component: `CardTitle`

`<div data-slot="card-title">`, class (verbatim):
```
"leading-none font-semibold"
```

### Component: `CardDescription`

`<div data-slot="card-description">`, class (verbatim):
```
"text-muted-foreground text-sm"
```

### Component: `CardAction`

`<div data-slot="card-action">`, class (verbatim):
```
"col-start-2 row-span-2 row-start-1 self-start justify-self-end"
```

### Component: `CardContent`

`<div data-slot="card-content">`, class: `"px-6"`.

### Component: `CardFooter`

`<div data-slot="card-footer">`, class (verbatim):
```
"flex items-center px-6 [.border-t]:pt-6"
```

## Step 5: Port `tabs.rs`

**Source:** `reference/paperclip/ui/src/components/ui/tabs.tsx`

File: `src/components/ui/tabs.rs`

### Component: `Tabs`

```rust
#[component]
pub fn Tabs(
    active_tab: Signal<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="tabs" data-orientation="horizontal">` with class (verbatim):
```
"group/tabs flex gap-2 flex-col"
```
merged with `&class`. Contains `{children}`.

### Component: `TabsList`

```rust
#[component]
pub fn TabsList(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="tabs-list" role="tablist">` with class (verbatim):
```
"bg-muted p-[3px] h-9 group/tabs-list text-muted-foreground inline-flex w-fit items-center justify-center"
```
merged with `&class`.

### Component: `TabsTrigger`

```rust
#[component]
pub fn TabsTrigger(
    value: String,
    active_tab: Signal<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<button data-slot="tabs-trigger" role="tab">` with:
- `aria-selected` set to `"true"` if `*active_tab.read() == value`, else `"false"`
- `data-state` set to `"active"` if selected, else `"inactive"`
- `onclick` sets `active_tab` to `value.clone()`
- Class (verbatim):
  ```
  "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:outline-ring text-foreground/60 hover:text-foreground dark:text-muted-foreground dark:hover:text-foreground relative inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap transition-[color,background-color,border-color,box-shadow] focus-visible:ring-[3px] focus-visible:outline-1 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 data-[state=active]:bg-background dark:data-[state=active]:text-foreground dark:data-[state=active]:border-input dark:data-[state=active]:bg-input/30 data-[state=active]:text-foreground data-[state=active]:shadow-sm"
  ```

### Component: `TabsContent`

```rust
#[component]
pub fn TabsContent(
    value: String,
    active_tab: Signal<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="tabs-content" role="tabpanel">` only when `*active_tab.read() == value`. Class: `"flex-1 outline-none"` merged with `&class`.

## Step 6: Port `tooltip.rs`

**Source:** `reference/paperclip/ui/src/components/ui/tooltip.tsx`

File: `src/components/ui/tooltip.rs`

### Component: `Tooltip`

```rust
#[component]
pub fn Tooltip(
    content: String,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Implement as a CSS-only hover tooltip using a relative container and absolute positioned content. Renders:

```
<div data-slot="tooltip" class="relative inline-flex group">
  {children}
  <div data-slot="tooltip-content"
       role="tooltip"
       class="bg-foreground text-background z-50 w-fit rounded-md px-3 py-1.5 text-xs text-balance absolute bottom-full left-1/2 -translate-x-1/2 mb-2 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity {class}">
    "{content}"
  </div>
</div>
```

## Step 7: Port `popover.rs`

**Source:** `reference/paperclip/ui/src/components/ui/popover.tsx`

File: `src/components/ui/popover.rs`

### Component: `Popover`

```rust
#[component]
pub fn Popover(
    open: Signal<bool>,
    children: Element,
) -> Element
```

Renders `<div data-slot="popover" class="relative inline-block">` wrapping `{children}`.

### Component: `PopoverTrigger`

```rust
#[component]
pub fn PopoverTrigger(
    open: Signal<bool>,
    children: Element,
) -> Element
```

Renders `<button data-slot="popover-trigger" onclick={toggle open}>` wrapping `{children}`.

### Component: `PopoverContent`

```rust
#[component]
pub fn PopoverContent(
    open: Signal<bool>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

When `*open.read()` is `false`, render `None`. When `true`, render `<div data-slot="popover-content">` with class (verbatim):
```
"bg-popover text-popover-foreground z-50 w-72 rounded-md border p-4 shadow-md outline-hidden absolute top-full mt-1"
```
merged with `&class`. Contains `{children}`.

## Step 8: Port `dropdown_menu.rs`

**Source:** `reference/paperclip/ui/src/components/ui/dropdown-menu.tsx`

File: `src/components/ui/dropdown_menu.rs`

### Component: `DropdownMenu`

```rust
#[component]
pub fn DropdownMenu(
    open: Signal<bool>,
    children: Element,
) -> Element
```

Renders `<div data-slot="dropdown-menu" class="relative inline-block">` wrapping `{children}`.

### Component: `DropdownMenuTrigger`

```rust
#[component]
pub fn DropdownMenuTrigger(
    open: Signal<bool>,
    children: Element,
) -> Element
```

Renders `<button data-slot="dropdown-menu-trigger" onclick={toggle open}>` wrapping `{children}`.

### Component: `DropdownMenuContent`

```rust
#[component]
pub fn DropdownMenuContent(
    open: Signal<bool>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

When `*open.read()` is `false`, render `None`. When `true`, render `<div data-slot="dropdown-menu-content" role="menu">` with class (verbatim):
```
"bg-popover text-popover-foreground z-50 min-w-[8rem] overflow-hidden rounded-md border p-1 shadow-md absolute top-full mt-1"
```
merged with `&class`. Contains `{children}`.

### Component: `DropdownMenuItem`

```rust
#[component]
pub fn DropdownMenuItem(
    #[props(default)] class: String,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element
```

Renders `<div data-slot="dropdown-menu-item" role="menuitem">` with:
- `data-disabled` set to `"true"` when disabled
- Class (verbatim):
  ```
  "focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
  ```
  merged with `&class`.

### Component: `DropdownMenuSeparator`

```rust
#[component]
pub fn DropdownMenuSeparator(
    #[props(default)] class: String,
) -> Element
```

Renders `<div data-slot="dropdown-menu-separator" role="separator">` with class (verbatim):
```
"bg-border -mx-1 my-1 h-px"
```

### Component: `DropdownMenuLabel`

```rust
#[component]
pub fn DropdownMenuLabel(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="dropdown-menu-label">` with class: `"px-2 py-1.5 text-sm font-medium"`.

## Step 9: Port `collapsible.rs`

**Source:** `reference/paperclip/ui/src/components/ui/collapsible.tsx`

File: `src/components/ui/collapsible.rs`

### Component: `Collapsible`

```rust
#[component]
pub fn Collapsible(
    open: Signal<bool>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="collapsible" data-state={if open "open" else "closed"}>` with `class` forwarded. Contains `{children}`.

### Component: `CollapsibleTrigger`

```rust
#[component]
pub fn CollapsibleTrigger(
    open: Signal<bool>,
    children: Element,
) -> Element
```

Renders `<button data-slot="collapsible-trigger" onclick={toggle open}>` wrapping `{children}`.

### Component: `CollapsibleContent`

```rust
#[component]
pub fn CollapsibleContent(
    open: Signal<bool>,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

When `*open.read()` is `false`, render `None`. When `true`, render `<div data-slot="collapsible-content">` with `class` forwarded. Contains `{children}`.

## Step 10: Port `command.rs`

**Source:** `reference/paperclip/ui/src/components/ui/command.tsx`

File: `src/components/ui/command.rs`

### Component: `Command`

```rust
#[component]
pub fn Command(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="command">` with class (verbatim):
```
"bg-popover text-popover-foreground flex h-full w-full flex-col overflow-hidden rounded-md"
```
merged with `&class`.

### Component: `CommandInput`

```rust
#[component]
pub fn CommandInput(
    #[props(default)] class: String,
    #[props(default)] placeholder: String,
    #[props(default)] value: String,
    #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element
```

Renders a wrapper `<div data-slot="command-input-wrapper" class="flex h-9 items-center gap-2 border-b px-3">` containing:
1. A search icon SVG: `<svg viewBox="0 0 24 24" class="size-4 shrink-0 opacity-50"><circle cx="11" cy="11" r="8" fill="none" stroke="currentColor" stroke-width="2"/><line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>`
2. An `<input data-slot="command-input">` with class (verbatim):
   ```
   "placeholder:text-muted-foreground flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
   ```
   merged with `&class`. Forwarding `placeholder`, `value`, `oninput`.

### Component: `CommandList`

```rust
#[component]
pub fn CommandList(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="command-list">` with class (verbatim):
```
"max-h-[300px] scroll-py-1 overflow-x-hidden overflow-y-auto"
```

### Component: `CommandEmpty`

```rust
#[component]
pub fn CommandEmpty(
    children: Element,
) -> Element
```

Renders `<div data-slot="command-empty" class="py-6 text-center text-sm">`.

### Component: `CommandGroup`

```rust
#[component]
pub fn CommandGroup(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="command-group">` with class: `"text-foreground overflow-hidden p-1"`.

### Component: `CommandItem`

```rust
#[component]
pub fn CommandItem(
    #[props(default)] class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element
```

Renders `<div data-slot="command-item">` with class (verbatim):
```
"data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
```

### Component: `CommandSeparator`

Renders `<div data-slot="command-separator" class="bg-border -mx-1 h-px">`.

## Step 11: Port `scroll_area.rs`

**Source:** `reference/paperclip/ui/src/components/ui/scroll-area.tsx`

File: `src/components/ui/scroll_area.rs`

### Component: `ScrollArea`

```rust
#[component]
pub fn ScrollArea(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<div data-slot="scroll-area">` with class (verbatim):
```
"relative flex flex-col overflow-hidden"
```
merged with `&class`, containing an inner viewport `<div data-slot="scroll-area-viewport" class="flex-1 min-h-0 w-full rounded-[inherit] overflow-y-auto overflow-x-hidden">` wrapping `{children}`.

## Step 12: Port `breadcrumb.rs`

**Source:** `reference/paperclip/ui/src/components/ui/breadcrumb.tsx`

File: `src/components/ui/breadcrumb.rs`

### Component: `Breadcrumb`

```rust
#[component]
pub fn Breadcrumb(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<nav data-slot="breadcrumb" aria-label="breadcrumb">` with `class` forwarded.

### Component: `BreadcrumbList`

```rust
#[component]
pub fn BreadcrumbList(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<ol data-slot="breadcrumb-list">` with class (verbatim):
```
"text-muted-foreground flex flex-wrap items-center gap-1.5 text-sm break-words sm:gap-2.5"
```

### Component: `BreadcrumbItem`

```rust
#[component]
pub fn BreadcrumbItem(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<li data-slot="breadcrumb-item">` with class: `"inline-flex items-center gap-1.5"`.

### Component: `BreadcrumbLink`

```rust
#[component]
pub fn BreadcrumbLink(
    #[props(default)] href: String,
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<a data-slot="breadcrumb-link" href="{href}">` with class (verbatim):
```
"hover:text-foreground transition-colors"
```

### Component: `BreadcrumbPage`

```rust
#[component]
pub fn BreadcrumbPage(
    #[props(default)] class: String,
    children: Element,
) -> Element
```

Renders `<span data-slot="breadcrumb-page" role="link" aria-disabled="true" aria-current="page">` with class (verbatim):
```
"text-foreground font-normal"
```

### Component: `BreadcrumbSeparator`

```rust
#[component]
pub fn BreadcrumbSeparator(
    #[props(default)] class: String,
) -> Element
```

Renders `<li data-slot="breadcrumb-separator" role="presentation" aria-hidden="true">` with class: `"[&>svg]:size-3.5"`. Contains a chevron-right SVG: `<svg viewBox="0 0 24 24" class="size-3.5"><polyline points="9 18 15 12 9 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>`.

### Component: `BreadcrumbEllipsis`

```rust
#[component]
pub fn BreadcrumbEllipsis(
    #[props(default)] class: String,
) -> Element
```

Renders `<span data-slot="breadcrumb-ellipsis" role="presentation" aria-hidden="true">` with class: `"flex size-9 items-center justify-center"`. Contains a more-horizontal SVG: `<svg viewBox="0 0 24 24" class="size-4"><circle cx="12" cy="12" r="1" fill="currentColor"/><circle cx="5" cy="12" r="1" fill="currentColor"/><circle cx="19" cy="12" r="1" fill="currentColor"/></svg>` and `<span class="sr-only">"More"</span>`.

## File Size Compliance

Estimated line counts per file:
- `dialog.rs`: ~140 lines
- `sheet.rs`: ~130 lines
- `card.rs`: ~90 lines
- `tabs.rs`: ~100 lines
- `tooltip.rs`: ~30 lines
- `popover.rs`: ~70 lines
- `dropdown_menu.rs`: ~120 lines
- `collapsible.rs`: ~55 lines
- `command.rs`: ~120 lines
- `scroll_area.rs`: ~25 lines
- `breadcrumb.rs`: ~100 lines

All under the 300-line limit.

## Definition of Done

1. `just diagnose` passes with zero errors and zero warnings for the `lx-desktop` crate
2. All 11 new files listed in the File Inventory exist at the specified paths
3. `src/components/ui/mod.rs` declares all 21 submodules (10 from Unit 1 + 11 from Unit 2)
4. Every component has the exact `data-slot` attribute values specified above
5. Every component has the exact Tailwind class strings specified above (verbatim)
6. All overlay components (`dialog`, `sheet`, `popover`, `dropdown_menu`, `collapsible`) use `Signal<bool>` for open/close state
7. ARIA attributes are present: `role="dialog"`, `aria-modal="true"` on dialog content; `role="menu"` / `role="menuitem"` on dropdown; `role="tablist"` / `role="tab"` / `role="tabpanel"` on tabs; `aria-label="breadcrumb"` on breadcrumb nav; `aria-hidden`, `aria-disabled`, `aria-current` on breadcrumb helpers; `role="tooltip"` on tooltip content
8. No file exceeds 300 lines
9. No `#[allow(...)]` attributes are used
10. No doc comments or code comments are present
