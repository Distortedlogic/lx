# WU-04: Chart semantic colors

## Fixes
- Fix 12: Add `--chart-1` through `--chart-5` CSS variables for chart semantic colors. These follow the shadcn/ui convention for themed chart palettes.

## Files Modified
- `crates/lx-desktop/src/tailwind.css` (196 lines)

## Preconditions
- `:root` block at lines 38-66 defines all CSS custom properties.
- Lines 63-65 already define chart-related variables (`--color-chart-axis`, `--color-chart-split`, `--color-chart-tooltip`) which are utility colors for chart chrome, not data series colors.
- The `@theme` block (lines 5-36) maps `--color-*` names to CSS variable values for Tailwind integration.

## Steps

### Step 1: Add --chart-1 through --chart-5 to :root
- Open `crates/lx-desktop/src/tailwind.css`
- At line 65 (after `--color-chart-tooltip: #191919;`), find:
```css
  --color-chart-tooltip: #191919;
}
```
- Replace with:
```css
  --color-chart-tooltip: #191919;

  --chart-1: #9cff93;
  --chart-2: #81ecff;
  --chart-3: #fcaf00;
  --chart-4: #ff7351;
  --chart-5: #c4b5fd;
}
```
- Why: Five semantic chart colors for data series. Colors reuse existing palette values where possible: `--chart-1` matches `--success`/`--primary` (green), `--chart-2` matches `--tertiary` (cyan), `--chart-3` matches `--warning` (amber), `--chart-4` matches `--error` (red-orange), `--chart-5` is a purple/violet for additional contrast.

### Step 2: Add Tailwind color mappings in @theme block
- At line 35 (after `--color-ring-offset: var(--surface);`), find:
```css
  --color-ring-offset: var(--surface);
}
```
- Replace with:
```css
  --color-ring-offset: var(--surface);

  --color-chart-1: var(--chart-1);
  --color-chart-2: var(--chart-2);
  --color-chart-3: var(--chart-3);
  --color-chart-4: var(--chart-4);
  --color-chart-5: var(--chart-5);
}
```
- Why: Registering in `@theme` makes these available as Tailwind utility classes (e.g., `text-chart-1`, `bg-chart-2`, `border-chart-3`).

## File Size Check
- `tailwind.css`: was 196 lines, now ~208 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compilation errors.
- Inspect the rendered CSS in browser devtools and confirm `--chart-1` through `--chart-5` are defined on `:root`.
- Confirm Tailwind classes like `text-chart-1` resolve correctly by temporarily adding one to a visible element.
