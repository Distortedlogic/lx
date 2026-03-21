# Dioxus Primitives Replacement Audit

Every item below is a binary check — a violation either exists or it does not. The audit identifies custom Dioxus components that duplicate functionality already provided by `dioxus-primitives`. Reference: `reference/dioxus_primitives/` contains the full source. The available primitives are: Accordion, AlertDialog, AspectRatio, Avatar, Calendar, Checkbox, Collapsible, ContextMenu, DatePicker, Dialog, DropdownMenu, HoverCard, Input, Label, Menubar, Navbar, Popover, Progress, RadioGroup, ScrollArea, Select, Separator, Sheet, Skeleton, Slider, Switch, Tabs, Textarea, Toast, Toggle, ToggleGroup, Toolbar, Tooltip.

---

## Discovery

- **Enumerate all custom components** — Find every `#[component]` function in the codebase. This is the candidate set.
  `rg '#\[component\]' --type rust crates/`
  `rg 'fn \w+\(.*\) -> Element' --type rust crates/`

- **Enumerate all dioxus-primitives already in use** — Determine which primitives the codebase already imports so you know what's available and what's missing.
  `rg 'dioxus.primitives|use.*primitives' --type rust crates/`

---

## Overlay & Dialog Patterns

- **Custom modal/dialog** — Detect custom components that implement modal/dialog behavior (overlay + centered content + escape-to-close + click-outside-to-close). Replace with `Dialog` or `AlertDialog` primitive. `AlertDialog` is for confirmations that require explicit user action (no click-outside dismiss). `Dialog` is for general modals.
  `rg -l 'modal|Modal' --type rust crates/`
  `rg 'fixed inset-0|z-50.*bg-black|overlay|backdrop' --type rust crates/`
  `rg 'onclick.*close|escape.*close|on_close' --type rust crates/`

- **Custom popover/floating content** — Detect custom components that show floating content anchored to a trigger element (tooltip-like but with richer content, click-triggered). Replace with `Popover` primitive.
  `rg -l 'popover|Popover|floating|Floating' --type rust crates/`
  `rg 'absolute.*top|absolute.*bottom|position.*absolute' --type rust crates/`

- **Custom tooltip** — Detect custom hover-triggered floating text/content. Replace with `Tooltip` primitive.
  `rg 'tooltip|Tooltip' --type rust crates/`
  `rg 'onmouseenter.*show|onmouseleave.*hide|hover.*tip' --type rust crates/`

- **Custom hover card** — Detect custom components that show a card/preview on hover with a delay. Replace with `HoverCard` primitive.
  `rg 'hover.*card|HoverCard|preview.*hover' --type rust crates/`

- **Custom sheet/drawer** — Detect custom slide-in panels from screen edges (sidebars, drawers, bottom sheets). Replace with `Sheet` primitive.
  `rg 'sheet|Sheet|drawer|Drawer|slide.*in|SlideIn' --type rust crates/`
  `rg 'translate-x-full|translate-y-full|-translate-x-full' --type rust crates/`

- **Custom context menu** — Detect custom right-click menus. Replace with `ContextMenu` primitive.
  `rg 'context.*menu|ContextMenu|oncontextmenu|right.click' --type rust crates/`

---

## Form & Input Patterns

- **Custom checkbox** — Detect custom toggle-with-checkmark components. Replace with `Checkbox` primitive.
  `rg 'checkbox|Checkbox' --type rust crates/`
  `rg 'checked.*unchecked|is_checked|toggle.*check' --type rust crates/`

- **Custom switch/toggle** — Detect custom boolean toggle components (sliding thumb on a track). Replace with `Switch` primitive. For press-to-toggle buttons (no sliding thumb), use `Toggle` primitive.
  `rg 'switch|Switch' --type rust crates/`
  `rg 'toggle.*button|ToggleButton|toggled|is_on' --type rust crates/`

- **Custom radio group** — Detect custom single-selection-from-a-group components. Replace with `RadioGroup` primitive.
  `rg 'radio|Radio' --type rust crates/`
  `rg 'selected.*option|single.*select|exclusive.*select' --type rust crates/`

- **Custom select/dropdown** — Detect custom dropdown select components (trigger button + dropdown list + single selection). Replace with `Select` primitive.
  `rg 'select.*dropdown|SelectDropdown|custom.*select|dropdown.*select' --type rust crates/`
  `rg 'is_open.*option|show_options|option_list' --type rust crates/`

- **Custom slider** — Detect custom range input components (track + draggable thumb). Replace with `Slider` primitive.
  `rg 'slider|Slider|range.*input|RangeInput' --type rust crates/`
  `rg 'thumb.*drag|track.*fill|onmousedown.*slide' --type rust crates/`

- **Custom input** — Detect custom text input wrapper components that add features like labels, error messages, or icons around a native input. Replace with `Input` primitive if it covers the use case.
  `rg '#\[component\]' -A5 --type rust crates/` then check for input wrappers.

- **Custom textarea** — Detect custom multi-line text input wrappers. Replace with `Textarea` primitive.
  `rg 'textarea|Textarea|TextArea|multiline.*input' --type rust crates/`

- **Custom label** — Detect custom label components that associate text with form controls. Replace with `Label` primitive.
  `rg 'Label|for.*input|htmlFor|label.*control' --type rust crates/`

- **Custom date picker** — Detect custom date selection components. Replace with `DatePicker` primitive (which composes `Calendar` + `Popover`).
  `rg 'date.*pick|DatePick|calendar.*select' --type rust crates/`

- **Custom calendar** — Detect custom calendar grid components. Replace with `Calendar` primitive.
  `rg 'calendar|Calendar' --type rust crates/`

---

## Layout & Navigation Patterns

- **Custom tabs** — Detect custom tab components (tab list + tab panels, one panel visible at a time). Replace with `Tabs` primitive.
  `rg 'tab.*list|TabList|tab.*panel|TabPanel|active.*tab|selected.*tab' --type rust crates/`
  `rg 'TabTrigger|TabContent|tab_index|current_tab' --type rust crates/`

- **Custom accordion** — Detect custom collapsible section components (header + expandable content, single or multiple open). Replace with `Accordion` primitive. For a single collapsible section, use `Collapsible` primitive.
  `rg 'accordion|Accordion' --type rust crates/`
  `rg 'collapsible|Collapsible|expandable|Expandable' --type rust crates/`
  `rg 'is_expanded|toggle.*expand|section.*open' --type rust crates/`

- **Custom navbar** — Detect custom navigation bar components. Replace with `Navbar` primitive (requires `router` feature).
  `rg 'navbar|Navbar|NavBar|nav.*bar' --type rust crates/`

- **Custom toolbar** — Detect custom toolbar components (horizontal row of action buttons/controls). Replace with `Toolbar` primitive.
  `rg 'toolbar|Toolbar|ToolBar|action.*bar' --type rust crates/`

- **Custom scroll area** — Detect custom scrollable container components with custom scrollbars. Replace with `ScrollArea` primitive.
  `rg 'scroll.*area|ScrollArea|custom.*scroll|scrollbar' --type rust crates/`

- **Custom separator** — Detect custom horizontal/vertical divider components. Replace with `Separator` primitive.
  `rg 'separator|Separator|Divider|divider|hr.*class' --type rust crates/`

---

## Display & Feedback Patterns

- **Custom progress bar** — Detect custom progress indicator components. Replace with `Progress` primitive.
  `rg 'progress|Progress|ProgressBar' --type rust crates/`
  `rg 'percent.*bar|fill.*width|completion.*bar' --type rust crates/`

- **Custom toast/notification** — Detect custom toast/snackbar/notification components. Replace with `Toast` primitive.
  `rg 'toast|Toast|snackbar|Snackbar|notification.*popup' --type rust crates/`

- **Custom avatar** — Detect custom user avatar components (image with fallback initials/icon). Replace with `Avatar` primitive.
  `rg 'avatar|Avatar' --type rust crates/`
  `rg 'initials|fallback.*image|user.*photo|profile.*pic' --type rust crates/`

- **Custom skeleton** — Detect custom loading placeholder/shimmer components. Replace with `Skeleton` primitive.
  `rg 'skeleton|Skeleton|shimmer|Shimmer|placeholder.*loading' --type rust crates/`

- **Custom aspect ratio container** — Detect custom aspect ratio wrappers. Replace with `AspectRatio` primitive.
  `rg 'aspect.*ratio|AspectRatio|padding.*bottom.*percent' --type rust crates/`

---

## Menu Patterns

- **Custom dropdown menu** — Detect custom dropdown menus triggered by a button (not select dropdowns — those are for form selection). Replace with `DropdownMenu` primitive.
  `rg 'dropdown.*menu|DropdownMenu|menu.*trigger|MenuTrigger' --type rust crates/`
  `rg 'menu.*item|MenuItem|action.*menu' --type rust crates/`

- **Custom menubar** — Detect custom horizontal menu bars (file/edit/view style). Replace with `Menubar` primitive.
  `rg 'menubar|Menubar|MenuBar|menu.*bar' --type rust crates/`

---

## Toggle Group Pattern

- **Custom toggle group** — Detect custom multi-toggle/segmented-control components (a group of toggles where one or more can be active). Replace with `ToggleGroup` primitive.
  `rg 'toggle.*group|ToggleGroup|segmented.*control|SegmentedControl|button.*group' --type rust crates/`

---

## Cross-Cutting Checks

- **Reimplemented primitive behaviors** — Detect custom implementations of behaviors that primitives handle internally: focus trapping, escape-to-close, click-outside-to-close, scroll locking, animated open/close transitions, ARIA attribute management. These are signs that a primitive should be used instead of a custom component.
  `rg 'focus.*trap|FocusTrap|trap.*focus' --type rust crates/`
  `rg 'click.*outside|ClickOutside|onblur.*close' --type rust crates/`
  `rg 'scroll.*lock|overflow.*hidden.*body|prevent.*scroll' --type rust crates/`
  `rg 'aria-|role=' --type rust crates/`

- **Partial primitive usage** — Detect components that import and use one part of a primitive but reimplement another part that the same primitive already provides. For example, using `Dialog::Overlay` but building custom content positioning instead of using `Dialog::Content`.
  `rg 'use.*primitives' --type rust crates/`
  Cross-reference: for each primitive imported, check if all sub-components are used or if some are reimplemented.
