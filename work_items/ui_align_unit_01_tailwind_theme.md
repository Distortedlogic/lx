# UI Alignment Unit 01: Tailwind v4 @theme Semantic Token Mappings

## Goal

Add Tailwind v4 `@theme` color/variable mappings to `tailwind.css` so that shadcn-style semantic class names (`bg-primary`, `text-muted-foreground`, `bg-card`, etc.) resolve to the existing Material Design `:root` CSS variables. Also fix `--radius-full` so `rounded-full` produces circles (9999px) instead of 0rem.

## File

`/home/entropybender/repos/lx/crates/lx-desktop/src/tailwind.css`

## Semantic Tokens Referenced by components/ui/ Files

Exhaustive list of every Tailwind semantic color token used across `components/ui/*.rs`:

| Token class pattern | CSS variable Tailwind expects | Used in |
|---|---|---|
| `bg-primary` | `--color-primary` | button.rs:33, badge.rs:20, checkbox.rs:34, avatar.rs:19 |
| `text-primary-foreground` | `--color-primary-foreground` | button.rs:33, badge.rs:20, checkbox.rs:34, avatar.rs:19 |
| `text-primary` | `--color-primary` | button.rs:42, badge.rs:27 |
| `bg-destructive` | `--color-destructive` | button.rs:35, badge.rs:23 |
| `ring-destructive` | `--color-destructive` | button.rs:29,35, badge.rs:16,23, input.rs:5, textarea.rs:5, select.rs:5, checkbox.rs:34 |
| `bg-background` | `--color-background` | button.rs:38, dialog.rs:30, sheet.rs:45, tabs.rs:50 |
| `bg-accent` | `--color-accent` | button.rs:38,41, badge.rs:25,26, dropdown_menu.rs:63, command.rs:104, skeleton.rs:7 |
| `text-accent-foreground` | `--color-accent-foreground` | button.rs:38,41, badge.rs:25,26, dropdown_menu.rs:63, command.rs:104 |
| `bg-secondary` | `--color-secondary` | button.rs:40, badge.rs:21 |
| `text-secondary-foreground` | `--color-secondary-foreground` | button.rs:40, badge.rs:21 |
| `bg-input` | `--color-input` | button.rs:38, tabs.rs:50, select.rs:5, input.rs:5, textarea.rs:5, checkbox.rs:34 |
| `border-input` | `--color-input` | button.rs:38, input.rs:5, textarea.rs:5, select.rs:5, checkbox.rs:34 |
| `bg-card` | `--color-card` | card.rs:12 |
| `text-card-foreground` | `--color-card-foreground` | card.rs:12 |
| `text-muted-foreground` | `--color-muted-foreground` | card.rs:53, dialog.rs:105, sheet.rs:121, breadcrumb.rs:24, command.rs:55,104, dropdown_menu.rs:63, tabs.rs:25,50, avatar.rs:17, input.rs:5, textarea.rs:5, select.rs:5 |
| `bg-muted` | `--color-muted` | tabs.rs:25, avatar.rs:17 |
| `text-foreground` | `--color-foreground` | badge.rs:25, breadcrumb.rs:50,64, command.rs:90, sheet.rs:110, tabs.rs:50, input.rs:5 |
| `border-border` | `--color-border` | badge.rs:25 |
| `bg-border` / `bg-separator` | `--color-border` | separator.rs:12 |
| `border-ring` / `ring-ring` | `--color-ring` | button.rs:29, badge.rs:16, input.rs:5, textarea.rs:5, select.rs:5, checkbox.rs:34, dialog.rs:37, sheet.rs:53, tabs.rs:50 |
| `ring-offset-background` | `--color-ring-offset` | dialog.rs:37, sheet.rs:53 |
| `bg-popover` | `--color-popover` | popover.rs:37, dropdown_menu.rs:38, command.rs:12 |
| `text-popover-foreground` | `--color-popover-foreground` | popover.rs:37, dropdown_menu.rs:38, command.rs:12 |
| `ring-background` | `--color-background` | avatar.rs:19 |

## Variable Mappings (semantic token -> existing :root var)

| Tailwind v4 theme variable | Value (mapping to existing :root var) |
|---|---|
| `--color-background` | `var(--surface)` |
| `--color-foreground` | `var(--on-surface)` |
| `--color-card` | `var(--surface-container)` |
| `--color-card-foreground` | `var(--on-surface)` |
| `--color-popover` | `var(--surface-container-high)` |
| `--color-popover-foreground` | `var(--on-surface)` |
| `--color-primary` | `var(--primary)` |
| `--color-primary-foreground` | `var(--on-primary)` |
| `--color-secondary` | `var(--surface-container-high)` |
| `--color-secondary-foreground` | `var(--on-surface)` |
| `--color-muted` | `var(--surface-container)` |
| `--color-muted-foreground` | `var(--on-surface-variant)` |
| `--color-accent` | `var(--surface-container-high)` |
| `--color-accent-foreground` | `var(--on-surface)` |
| `--color-destructive` | `var(--error)` |
| `--color-border` | `var(--outline-variant)` |
| `--color-input` | `var(--outline-variant)` |
| `--color-ring` | `var(--primary)` |
| `--color-ring-offset` | `var(--surface)` |

## Exact Change

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/tailwind.css`

**old_string:**
```
@theme {
  --font-display: 'Space Grotesk', sans-serif;
  --font-body: 'Inter', sans-serif;
  --radius-sm: 0rem;
  --radius: 0rem;
  --radius-md: 0rem;
  --radius-lg: 0rem;
  --radius-xl: 0rem;
  --radius-2xl: 0rem;
  --radius-3xl: 0rem;
}
```

**new_string:**
```
@theme {
  --font-display: 'Space Grotesk', sans-serif;
  --font-body: 'Inter', sans-serif;
  --radius-sm: 0rem;
  --radius: 0rem;
  --radius-md: 0rem;
  --radius-lg: 0rem;
  --radius-xl: 0rem;
  --radius-2xl: 0rem;
  --radius-3xl: 0rem;
  --radius-full: 9999px;

  --color-background: var(--surface);
  --color-foreground: var(--on-surface);
  --color-card: var(--surface-container);
  --color-card-foreground: var(--on-surface);
  --color-popover: var(--surface-container-high);
  --color-popover-foreground: var(--on-surface);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--on-primary);
  --color-secondary: var(--surface-container-high);
  --color-secondary-foreground: var(--on-surface);
  --color-muted: var(--surface-container);
  --color-muted-foreground: var(--on-surface-variant);
  --color-accent: var(--surface-container-high);
  --color-accent-foreground: var(--on-surface);
  --color-destructive: var(--error);
  --color-border: var(--outline-variant);
  --color-input: var(--outline-variant);
  --color-ring: var(--primary);
  --color-ring-offset: var(--surface);
}
```

## Verification

After applying the change, every Tailwind semantic class in `components/ui/*.rs` will resolve through the `@theme` block to the Material Design `:root` variables. The `rounded-full` class will produce `border-radius: 9999px` (circles), while all other radii remain `0rem` (sharp corners).
