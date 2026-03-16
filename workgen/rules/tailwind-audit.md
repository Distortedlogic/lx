# Tailwind CSS Audit

Every item below is a binary check — a violation either exists or it does not. The audit covers all `.css` files and all `.rs` files containing RSX `class:` attributes.

Reference: `reference/tailwindcss.com/src/docs/` contains the full Tailwind documentation as MDX files. Key files: `upgrade-guide.mdx`, `theme.mdx`, `adding-custom-styles.mdx`.

---

## Custom Style Classification

All custom CSS must be placed in exactly one of four locations per the Tailwind v4 docs at `reference/tailwindcss.com/src/docs/adding-custom-styles.mdx` and `reference/tailwindcss.com/src/docs/theme.mdx`.

- **`@theme`** — design tokens (colors as `--color-*`, fonts as `--font-*`, animations as `--animate-*` with nested `@keyframes`, spacing, breakpoints). Auto-generates utility classes. Custom colors MUST use this. Example: `--color-accent-info: oklch(0.707 0.165 254.624)` generates text-accent-info, bg-accent-info, border-accent-info, fill-accent-info, stroke-accent-info, ring-accent-info automatically.

- **`@layer base`** — element-level defaults (body, button cursor). Applied to HTML elements/selectors, not class names.

- **`@layer components`** — multi-property named classes for UI components (card, btn-primary, dialog-overlay, badge-*, etc.). Overridable by utilities. No inline variant prefix support in markup. Hover/focus/disabled states use CSS nesting or `@variant` inside the definition. Use dot-prefixed class names.

- **`@utility`** — single-purpose custom utilities not in Tailwind (scrollbar-hide), or classes that use responsive variant prefixes in their `@apply` body. Atomic, one concern per utility. Use `@utility name { }` syntax.

**How to decide:** If the class sets 3+ CSS properties or combines multiple visual concerns (color + spacing + border + radius), it is a component class (`@layer components`). Named parts of a component system (badge-\*, accordion-\*, tab-trigger-\*, slider-\*) also belong in `@layer components` regardless of property count. If it sets a single CSS property or toggles a single behavior and is not part of a component system, it is a utility (`@utility`). If it uses responsive/state variant prefixes in its `@apply` body, it must be `@utility`. If it defines a reusable value (color, font, animation), it is a design token (`@theme`).

---

## High Frequency Checks

Violations commonly introduced by AI agents and during rapid development. Run these first.

### High Fuckup Chance

Items AI agents get wrong by default due to training data bias, stale v3 knowledge, or laziness shortcuts.

- **v3 `@tailwind` directives instead of `@import`** — Detect `@tailwind base`, `@tailwind components`, or `@tailwind utilities` in CSS files. Fix: replace with `@import "tailwindcss"`.
  `rg '@tailwind' crates/ --glob '*.css'`

- **Component classes in `@utility` instead of `@layer components`** — Detect `@utility` definitions whose body sets 3+ CSS properties (via `@apply` with 3+ classes, or 3+ direct declarations) and does not use responsive/state variant prefixes in its `@apply` body. Fix: move to `@layer components { .name { ... } }` with dot-prefixed class name.
  `rg '@utility' crates/ --glob '*.css'`

- **Custom colors as `@utility` wrappers instead of `@theme` variables** — Detect `@utility` definitions whose body is solely `@apply` with a single color utility. Fix: define the color in `@theme` as a `--color-*` variable using oklch. For paired utilities (text shade + bg shade), define two theme variables: `--color-accent-info` for the text shade (400 level oklch) and `--color-accent-info-bg` for the bg shade (500 level oklch).
  `rg '@utility.*\{' crates/ --glob '*.css' -A1`

- **Wrong shadow/radius/blur scale** — The scale shifted down in v4. AI agents trained on v3 emit the old names. `shadow-sm` in v3 = `shadow-xs` in v4. `shadow` (bare) in v3 = `shadow-sm` in v4. Same pattern for `rounded`, `blur`, `drop-shadow`, `backdrop-blur`. Fix: when introducing NEW classes, use the correct scale names. Flag bare `shadow`, `rounded`, `blur`, `drop-shadow`, `backdrop-blur` without size suffix — these map to `-sm` now (different visual from v3).
  `rg 'shadow-sm|rounded-sm|blur-sm|drop-shadow-sm|backdrop-blur-sm' --type rust crates/`

- **`outline-none` instead of `outline-hidden`** — Detect `outline-none` in class attributes and CSS. In v4, `outline-none` sets `outline-style: none` (removes forced-colors-mode accessibility outline). The old invisible-outline behavior is now `outline-hidden`. Fix: replace with `outline-hidden`.
  `rg 'outline-none' --type rust crates/`
  `rg 'outline-none' crates/ --glob '*.css'`

- **`ring` expecting 3px width** — `ring` is 1px in v4 (was 3px in v3). Used in 43 files in this codebase. Detect bare `ring` without a width suffix. Fix: use `ring-3` for old 3px, or explicit `ring-1`, `ring-2`.
  `rg '"[^"]*\bring\b[^-]' --type rust crates/`

- **Raw palette colors instead of semantic tokens** — This codebase defines semantic colors: `background`, `foreground`, `card`, `card-foreground`, `popover`, `popover-foreground`, `primary`, `primary-foreground`, `secondary`, `secondary-foreground`, `muted`, `muted-foreground`, `accent`, `accent-foreground`, `destructive`, `destructive-foreground`, `success`, `success-foreground`, `warning`, `warning-foreground`, `border`, `input`, `ring`. AI agents default to raw palette colors (`bg-red-500`, `text-gray-400`) instead of semantic tokens (`bg-destructive`, `text-muted-foreground`). Fix: use semantic names. Exception: chart accent utilities (`accent-info`, `accent-purple`, `accent-cyan`, `accent-orange`).
  `rg '(bg|text|border)-(red|blue|green|gray|slate|zinc|neutral|stone|amber|yellow|emerald|teal|cyan|sky|indigo|violet|purple|fuchsia|pink|rose)-[0-9]' --type rust crates/`

- **Not using established custom classes** — This codebase defines custom classes via `@theme` color variables, `@layer components` component classes, and `@utility` utilities. AI agents recompose the constituent classes instead of using the custom class name. Fix: use the custom class.
  `@layer components` classes: card, card-sm, card-inner, btn-primary, btn-secondary, btn-ghost, btn-destructive, input, number-input, select-trigger, select-option, select-list, tab-trigger, tab-trigger-sm, tab-trigger-active, dialog-overlay, sheet-overlay, dialog-content, accordion-item, accordion-trigger, accordion-content, empty-state, card-header, section-heading, subsection-heading, stat-label, chart-placeholder, gen-label, tab-bar, progress-track, form-label, section-bar, tooltip-content, tooltip-wrapper, switch-thumb, slider-root, slider-track, slider-range, slider-thumb, badge-primary, badge-purple, badge-green, badge-amber, badge-orange, badge-teal, badge-yellow, badge-violet, badge-muted, badge-none.
  `@utility` classes: scrollbar-hide, btn-loading, stat-grid, stat-grid-5, chart-height, metric-grid, chart-compact.
  `@theme` color variables (`--color-*`): accent-info, accent-info-bg, accent-purple, accent-purple-bg, accent-cyan, accent-cyan-bg, accent-orange, accent-orange-bg, accent-amber, accent-amber-bg, accent-emerald, accent-emerald-bg, accent-rose, accent-rose-bg, accent-violet, accent-violet-bg, accent-teal, accent-teal-bg, accent-yellow, accent-yellow-bg, accent-green, accent-green-bg.
  `rg 'bg-card text-card-foreground rounded-lg' --type rust crates/`
  `rg 'bg-primary text-primary-foreground rounded-lg' --type rust crates/`
  `rg 'fixed inset-0 bg-black/50' --type rust crates/`

- **String interpolation mixed with static classes** — (Cross-reference: `dioxus-audit.md`) Detect RSX `class:` attributes that mix `{variable}` interpolation with static Tailwind classes in a single string. 20+ files currently have this. Fix: split into separate `class:` attributes — one for all static classes, one per interpolated value.
  `rg 'class:\s*"[^"]*\{[^}]+\}[^"]*"' --type rust crates/`

- **Conditional classes hoisted into `let` bindings instead of inline `class: if`** — Detect `let` bindings that compute class strings via `if/else` (e.g., `let nav_width = if collapsed() { "w-12" } else { "w-64" };`) and are then interpolated into `class: "{var}"`. Dioxus supports `class: if cond { "classes-a" } else { "classes-b" }` directly in RSX, which is shorter, keeps styling co-located with markup, and avoids string interpolation. Fix: remove the `let` bindings and use inline `class: if` attributes. When multiple `let` bindings share the same condition, merge them into a single `class: if` with the classes combined.
  `rg 'let \w+ = if .* \{ "[\w\s-]+" \} else \{ "[\w\s-]+" \}' --type rust crates/`

- **`dark:` prefix used** — Dark-only codebase. Any `dark:` variant or `prefers-color-scheme` query is wrong. Fix: remove.
  `rg 'dark:' --type rust crates/`
  `rg 'dark:|prefers-color-scheme' crates/ --glob '*.css'`

- **Gray or blue tones in dark backgrounds** — The codebase avoids gray (`slate`, `gray`, `zinc`, `neutral`, `stone`) and blue-tinted backgrounds because they produce an undesirable dark-blue appearance. Background colors must use the warm neutral tones defined in `@theme` (`--color-background: #0a0a0a`, `--color-card: #171717`, `--color-secondary: #262626`, `--color-muted: #262626`, `--color-accent: #404040`). Fix: use semantic background tokens. Never introduce `bg-slate-*`, `bg-gray-*`, `bg-zinc-*`, `bg-neutral-*`, or `bg-blue-*` for UI backgrounds.
  `rg 'bg-(slate|gray|zinc|neutral|stone|blue)-' --type rust crates/`

- **Bare `border` without color** — `border` defaults to `currentColor` in v4 (was `gray-200` in v3). Detect `border` without an accompanying `border-*` color class. Fix: always specify border color (`border border-border`).
  `rg 'class:.*"[^"]*\bborder\b' --type rust crates/`

- **`focus:` instead of `focus-visible:`** — Detect `focus:ring`, `focus:outline`, `focus:border`. Using `focus:` shows focus ring on mouse click. Fix: use `focus-visible:` for keyboard-only focus indicators. Exception: `focus:outline-hidden` and `focus:border-ring` on inputs are intentional (inputs always show focus).
  `rg 'focus:' --type rust crates/`

- **Duplicated class combinations not extracted** — The same combination of 4+ utility classes appears in 3+ RSX locations. Fix: create a `@utility` in `tailwind.css` or `primitives.css`.
  `rg -o 'class: "[^"]*"' --type rust --no-filename crates/ | sort | uniq -c | sort -rn | head -30`

- **Redundant `cursor-pointer` on buttons** — Base styles set `cursor: pointer` on `button:not(:disabled)` and `[role="button"]:not(:disabled)`. Adding `cursor-pointer` to `<button>` is redundant. Fix: remove from buttons. Only use on non-button interactive elements.
  `rg 'button.*cursor-pointer|cursor-pointer.*button' --type rust crates/`

### Low Fuckup Chance

Items AI agents are less likely to get wrong but still need checking.

- **v3 deprecated opacity utilities** — `bg-opacity-*`, `text-opacity-*`, `border-opacity-*`, `divide-opacity-*`, `ring-opacity-*`, `placeholder-opacity-*`. Fix: use opacity modifier syntax (`bg-black/50`).
  `rg '(bg|text|border|divide|ring|placeholder)-opacity-' --type rust crates/`

- **v3 deprecated utility names** — `flex-shrink-*` → `shrink-*`, `flex-grow-*` → `grow-*`, `overflow-ellipsis` → `text-ellipsis`, `decoration-slice` → `box-decoration-slice`, `decoration-clone` → `box-decoration-clone`.
  `rg 'flex-shrink|flex-grow|overflow-ellipsis|decoration-slice|decoration-clone' --type rust crates/`

- **v3 `theme()` with dot notation** — Detect `theme(colors.red.500)` or `theme(screens.xl)`. Fix: use `var(--color-red-500)` directly. If `theme()` is needed (media queries), use `theme(--breakpoint-xl)`.
  `rg 'theme\(' crates/ --glob '*.css'`

- **v3 CSS variable shorthand with square brackets** — Detect `bg-[--var]`, `text-[--var]`. Fix: use parentheses `bg-(--var)`.
  `rg '\[--([\w-]+)\]' --type rust crates/`

- **v3 `!` at beginning for important modifier** — Detect `!bg-`, `!text-`, `!p-`. Fix: place `!` at end: `bg-red-500!`.
  `rg '"[^"]*\![a-z]' --type rust crates/`

- **v3 `@screen` directive** — Fix: use `@media (width >= ...)` or responsive variant prefixes.
  `rg '@screen' crates/ --glob '*.css'`

- **v3 variant stacking order** — v4 applies left-to-right. Detect `first:*:`, `last:*:`. Fix: reverse to `*:first:`, `*:last:`.
  `rg 'first:\*:|last:\*:' --type rust crates/`

- **Missing accessibility state variants** — Interactive elements with `hover:` but missing `focus-visible:` or `disabled:`. Fix: pair all three on interactive elements.
  `rg 'hover:' --type rust crates/`

- **Arbitrary pixel values where spacing scale works** — `p-[13px]`, `m-[7px]`, `gap-[9px]` where a scale value fits. Fix: use scale values.
  `rg '\b(p|m|gap|space|w|h|top|right|bottom|left|inset)-\[\d+px\]' --type rust crates/`

- **Arbitrary z-index in RSX** — Detect `z-[9999]`, `z-[100]`. Fix: use `z-50` max in RSX. Higher values belong in CSS `@utility` or `@layer base`.
  `rg 'z-\[' --type rust crates/`

---

## Low Frequency Checks

Structural or rare violations. Run after high frequency checks.

### High Fuckup Chance

- **`@apply` in scoped css_module files without `@reference`** — Detect `@apply` in CSS files used with `css_module!()` or standalone component CSS that isn't imported via the main `tailwind.css` entry point. These files don't have access to theme variables or custom utilities without `@reference`. Fix: add `@reference "../../tailwind.css"` at top.
  `rg '@apply' crates/ --glob '*.css'`
  Cross-reference: is the file imported via the main CSS entry point? If not, it needs `@reference`.

- **`style` attribute for static values** — Detect `style:` with hardcoded static values that could be Tailwind classes (`style: "width: 200px"` → `w-[200px]`). Fix: use Tailwind classes. Reserve `style:` only for dynamic runtime-computed values (percentages from data, calculated positions). 18 files currently use `style:`.
  `rg 'style:' --type rust crates/`

- **`@keyframes` outside `@theme` for animation utilities** — Detect `@keyframes` outside `@theme` that have a matching `--animate-*` variable. Fix: nest `@keyframes` inside `@theme`. Exception: always-included keyframes (dialog animations, toast animations) can stay outside.
  `rg '@keyframes' crates/ --glob '*.css'`

- **`space-y-*`/`space-x-*` where `gap` fits** — v4 changed the `space-*` selector to `:not(:last-child)` with `margin-bottom`. Detect `space-*` on flex/grid containers where `gap-*` would be simpler. Fix: use `gap-*`. 108 occurrences in codebase vs 172 `gap-*` — both are used but `gap` is preferred.
  `rg 'space-(x|y)-' --type rust crates/`

### Low Fuckup Chance

- **Hex/RGB colors in `@theme` instead of oklch** — Tailwind v4 default palette uses oklch. Custom colors in `@theme` using hex lack perceptual uniformity and consistent opacity modification. Fix: convert to oklch. Scope: only `@theme` block.
  `rg '#[0-9a-fA-F]{3,8}' crates/ --glob 'tailwind.css'`

- **`@utility` uses `@apply` correctly** — Verify `@apply` only exists inside `@utility` blocks and `@layer base` blocks. Detect stray `@apply` outside these contexts. Fix: move into proper block.
  `rg '@apply' crates/ --glob '*.css'`

- **Toast styles exemption** — `primitives.css` contains raw CSS for toast components (`.toast-container`, `.toast-list`, `.toast-item`, `.toast`, `.toast-content`, `.toast-title`, `.toast-description`, `.toast-close`). These use raw CSS due to complex `calc()` with CSS variables (`--toast-count`, `--toast-index`). Do NOT flag — intentionally raw CSS.

- **Unnecessary CSS file proliferation** — Detect CSS files imported into the main tailwind.css via `@import` whose content could be placed directly in tailwind.css in the appropriate section (`@theme`, `@layer components`, `@layer base`). Fix: merge and remove the separate file. Exception: files with complex raw CSS benefiting from isolation (primitives.css with toast CSS, scoped CSS modules).
  `rg '@import' crates/ --glob 'tailwind.css'`

---

## Dark Mode & Color Conventions

This codebase is dark-first, dark-by-default, and dark-only. There is no light mode. There never will be. Do not introduce `dark:` variants, `prefers-color-scheme` queries, light/dark toggles, or conditional theming of any kind. All colors in `@theme` are the only colors — they are not "dark mode" overrides of something else.

No grays. No dark blues. Backgrounds are warm black/neutral tones, never gray-family (`slate`, `gray`, `zinc`, `neutral`, `stone`) or blue-family. The palette is masculine — warm, bold, saturated, high-contrast on deep black. No pastels, no cool tones, no washed-out colors, ever.

- **No gray or blue background tones** — Background colors must use the warm neutral tones from `@theme`. The defined background scale is: `#0a0a0a` (background), `#171717` (card), `#262626` (secondary/muted), `#404040` (accent). Never introduce `bg-slate-*`, `bg-gray-*`, `bg-zinc-*`, `bg-neutral-*`, `bg-stone-*`, or `bg-blue-*` for any UI element.

- **Warm, bold, masculine palette** — Primary is amber/orange (`#d97706`). Destructive is red (`#ef4444`). Success is green (`#22c55e`). Warning is amber (`#f59e0b`). These are high-contrast, saturated colors on deep black backgrounds. New semantic colors must follow this pattern — warm, saturated, strong contrast. Avoid pastels, cool-toned grays, muted blues, or washed-out colors.

- **Semantic tokens only for UI** — All UI colors must use semantic tokens (`bg-card`, `text-foreground`, `border-border`, `text-primary`, `bg-destructive`, etc.). Raw palette colors are only acceptable in chart/data-visualization contexts and accent utility classes.

- **Custom colors via `@theme` variables** — All custom colors (accent colors for charts, badge colors, status indicators) must be defined as `--color-*` variables in the `@theme` block using oklch values. This auto-generates utility classes in all color-related namespaces (text-\*, bg-\*, border-\*, fill-\*, stroke-\*, ring-\*). Never define custom colors as `@utility` wrappers. Reference `reference/tailwindcss.com/src/docs/colors.mdx` section "Customizing your colors."

---

## CSS Authoring in `@layer components`

Component classes should use raw CSS with theme variable references per `reference/tailwindcss.com/src/docs/styling-with-utility-classes.mdx` (the `btn-primary` `@layer components` example). Theme variable reference patterns:

- Colors: `var(--color-card)`, `var(--color-primary)`
- Spacing: `--spacing(4)`, `--spacing(2)`
- Radius: `var(--radius-lg)`, `var(--radius-md)`
- Shadows: `var(--shadow-md)`, `var(--shadow-sm)`
- Fonts: `var(--font-weight-semibold)`, `var(--text-sm)`
- Opacity: `--alpha(var(--color-primary) / 20%)`

- **`@apply` usage** — `@apply` is acceptable inside `@layer components` when it replaces 4+ lines of raw CSS, but for 1-2 properties use raw CSS with theme variable references.

- **Hover/focus/disabled states** — States inside component classes must use CSS nesting: `&:hover` wrapped in `@media (hover: hover)` for hover, `&:focus-visible` for keyboard focus, `&:disabled` for disabled state.

---

## Scoped CSS Modules (`css_module!`)

Dioxus supports scoped CSS via the `css_module!` macro (from manganis). This section covers correct usage, integration with Tailwind, and gotchas.

### How It Works

The `css_module!` macro takes a CSS file path, parses all `.class` and `#id` selectors, appends a deterministic hash suffix (e.g., `.button` → `.button-e1e1ad32`), and generates a Rust struct with `const` fields for each scoped name. The stylesheet is lazily injected into the document on first field access via `Deref`.

```rust
css_module!(Styles = "/assets/styles.css");

rsx! {
    div { class: Styles::container,
        button { class: Styles::button, "Click me" }
    }
}
```

### Naming Convention

CSS class/ID names are converted to `snake_case` for Rust field access:

- `my-class` → `Styles::my_class`
- `fooBar` → `Styles::foo_bar`
- If both `.button` and `#button` exist, the class gets a `_class` suffix: `Styles::button` (ID), `Styles::button_class` (class)

### Visibility

The struct visibility is controlled by the macro:

- `css_module!(Styles = "...")` — private
- `css_module!(pub Styles = "...")` — public
- `css_module!(pub(crate) Styles = "...")` — crate-visible

### Integration with Tailwind

- **Do NOT use `css_module!` for files that contain Tailwind utility classes** — Tailwind utilities are global by design. Scoping them with hash suffixes breaks their purpose. Use `asset!()` / `document::Stylesheet` for global Tailwind CSS.
- **Do NOT use `@apply` inside `css_module!` files without `@reference`** — Scoped CSS files are processed independently. They do not have access to Tailwind theme variables, custom utilities, or variants unless you add `@reference "../../tailwind.css"` at the top of the CSS file.
- **Use `css_module!` for component-scoped styles that cannot be expressed as Tailwind utilities** — Complex animations, CSS variable calculations, intricate pseudo-element chains, or third-party component overrides.
- **Prefer `@utility` over `css_module!` for reusable component styles** — If a style set will be used across multiple components, define it as a `@utility` in `tailwind.css`. Use `css_module!` only for truly component-private styles.

### `:global()` Syntax

Use `:global(.class-name)` to opt out of scoping for specific selectors. The class name inside `:global()` will NOT get a hash suffix. Use this for:

- Targeting third-party component classes
- Targeting Tailwind utility classes from within a css_module file
- Shared state classes set by JavaScript

```css
:global(.global-class) {
  color: red;
  font-weight: bold;
}
```

### Gotchas and Limitations

- **Media query idents not collected** — Class/ID selectors defined ONLY inside `@media` queries are not extracted by the parser. Workaround: add an empty block for the selector outside the media query.
  ```css
  .responsive-thing {
  }
  @media (min-width: 768px) {
    .responsive-thing {
      padding: 2rem;
    }
  }
  ```
- **File path resolution** — Absolute paths (starting with `/`) resolve relative to `CARGO_MANIFEST_DIR`. Relative paths (starting with `.`) require Rust 1.88+. Supports `concat!()` and `env!()` in paths.
- **Lazy injection** — The stylesheet link is injected into the document only when the first field is accessed, not at import/macro expansion time. This means styles won't apply to elements rendered before any field access.
- **No composition with Tailwind `class:` merging** — When using `css_module!` fields in `class:` attributes, they produce scoped class names. You can combine them with Tailwind utilities using multiple `class:` attributes:
  ```rust
  div {
      class: Styles::container,
      class: "flex items-center gap-4",
  }
  ```

### CSS Module Audit Checks

- **`css_module!` used for Tailwind utility styles** — Detect `css_module!` files that contain Tailwind utility classes or `@apply` directives without `@reference`. Fix: either add `@reference` or move styles to a `@utility` in the global CSS.
  `rg 'css_module!' --type rust crates/`
  For each match: read the referenced CSS file and check for `@apply` without `@reference`.

- **`css_module!` where `@utility` suffices** — Detect `css_module!` files whose styles are used by multiple components. Fix: extract to `@utility` in `tailwind.css`.
  `rg 'css_module!' --type rust crates/`
  Cross-reference: is the struct imported/used in more than one component file?

- **Missing `:global()` for shared classes** — Detect CSS module files that target classes set externally (by JavaScript, Tailwind, or parent components) without `:global()`. Fix: wrap in `:global()`.

- **Media query selectors not extracted** — Detect CSS module files with selectors that only appear inside `@media` queries. Fix: add an empty block for the selector outside the media query.
  `rg '@media' crates/ --glob '*.module.css'`
