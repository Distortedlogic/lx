# WU-13: Font Bundling

## Fixes
- Fix 1: Replace Google Fonts CDN link with locally bundled font files to eliminate external network dependency

## Files Modified
- `crates/lx-desktop/src/app.rs` (42 lines)
- `crates/lx-desktop/assets/fonts/` (new directory, new font files)
- `crates/lx-desktop/assets/fonts.css` (new file)

## Preconditions
- `app.rs` line 20 contains the Google Fonts CDN `document::Link` element loading Space Grotesk, Inter, JetBrains Mono, and Material Symbols Outlined
- `tailwind.css` at lines 6-7 references `'Space Grotesk'` and `'Inter'` as `--font-display` and `--font-body`
- `assets/` directory exists at `crates/lx-desktop/assets/` and currently contains `charts.js`, `echarts-5.5.1.min.js`, `tailwind.css`, `widget-bridge.js`
- No `fonts/` subdirectory exists yet
- The CDN URL in `app.rs` line 20 is: `https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0&display=swap`

## Font Files Required

Four font families. All downloaded via `google-webfonts-helper` API (provides direct woff2 download links) or `fontsource` npm packages. The target filenames below are what the `@font-face` declarations reference; if a download produces different filenames, rename to match.

### 1. Space Grotesk (weights: 300, 400, 500, 600, 700)

Download via google-webfonts-helper zip:
```bash
curl -L -o /tmp/space-grotesk.zip "https://gwfh.mranftl.com/api/fonts/space-grotesk?download=zip&subsets=latin&variants=300,400,500,600,700"
unzip /tmp/space-grotesk.zip -d /tmp/space-grotesk
```
Copy and rename to target filenames:
```bash
cp /tmp/space-grotesk/space-grotesk-v17-latin-300.woff2 crates/lx-desktop/assets/fonts/SpaceGrotesk-Light.woff2
cp /tmp/space-grotesk/space-grotesk-v17-latin-regular.woff2 crates/lx-desktop/assets/fonts/SpaceGrotesk-Regular.woff2
cp /tmp/space-grotesk/space-grotesk-v17-latin-500.woff2 crates/lx-desktop/assets/fonts/SpaceGrotesk-Medium.woff2
cp /tmp/space-grotesk/space-grotesk-v17-latin-600.woff2 crates/lx-desktop/assets/fonts/SpaceGrotesk-SemiBold.woff2
cp /tmp/space-grotesk/space-grotesk-v17-latin-700.woff2 crates/lx-desktop/assets/fonts/SpaceGrotesk-Bold.woff2
```
The version number in the filename (e.g., `v17`) may differ. Use a glob: `cp /tmp/space-grotesk/space-grotesk-*-latin-300.woff2 ...`

### 2. Inter (weights: 300, 400, 500, 600, 700)

```bash
curl -L -o /tmp/inter.zip "https://gwfh.mranftl.com/api/fonts/inter?download=zip&subsets=latin&variants=300,400,500,600,700"
unzip /tmp/inter.zip -d /tmp/inter
cp /tmp/inter/inter-*-latin-300.woff2 crates/lx-desktop/assets/fonts/Inter-Light.woff2
cp /tmp/inter/inter-*-latin-regular.woff2 crates/lx-desktop/assets/fonts/Inter-Regular.woff2
cp /tmp/inter/inter-*-latin-500.woff2 crates/lx-desktop/assets/fonts/Inter-Medium.woff2
cp /tmp/inter/inter-*-latin-600.woff2 crates/lx-desktop/assets/fonts/Inter-SemiBold.woff2
cp /tmp/inter/inter-*-latin-700.woff2 crates/lx-desktop/assets/fonts/Inter-Bold.woff2
```

### 3. JetBrains Mono (weights: 400, 500)

```bash
curl -L -o /tmp/jetbrains-mono.zip "https://gwfh.mranftl.com/api/fonts/jetbrains-mono?download=zip&subsets=latin&variants=regular,500"
unzip /tmp/jetbrains-mono.zip -d /tmp/jetbrains-mono
cp /tmp/jetbrains-mono/jetbrains-mono-*-latin-regular.woff2 crates/lx-desktop/assets/fonts/JetBrainsMono-Regular.woff2
cp /tmp/jetbrains-mono/jetbrains-mono-*-latin-500.woff2 crates/lx-desktop/assets/fonts/JetBrainsMono-Medium.woff2
```

### 4. Material Symbols Outlined (weight 400, opsz 24, FILL 0, GRAD 0)

The CDN URL in `app.rs` line 20 returns a CSS stylesheet that contains a `src: url(...)` pointing to the actual woff2 on `fonts.gstatic.com`. Extract and download it:
```bash
curl -s "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0" \
  | grep -oP 'url\(\K[^)]+\.woff2' \
  | head -1 \
  | xargs -I{} curl -L -o crates/lx-desktop/assets/fonts/MaterialSymbolsOutlined.woff2 "{}"
```

### Verification of all target filenames
After download and rename, these exact files must exist:
- `crates/lx-desktop/assets/fonts/SpaceGrotesk-Light.woff2`
- `crates/lx-desktop/assets/fonts/SpaceGrotesk-Regular.woff2`
- `crates/lx-desktop/assets/fonts/SpaceGrotesk-Medium.woff2`
- `crates/lx-desktop/assets/fonts/SpaceGrotesk-SemiBold.woff2`
- `crates/lx-desktop/assets/fonts/SpaceGrotesk-Bold.woff2`
- `crates/lx-desktop/assets/fonts/Inter-Light.woff2`
- `crates/lx-desktop/assets/fonts/Inter-Regular.woff2`
- `crates/lx-desktop/assets/fonts/Inter-Medium.woff2`
- `crates/lx-desktop/assets/fonts/Inter-SemiBold.woff2`
- `crates/lx-desktop/assets/fonts/Inter-Bold.woff2`
- `crates/lx-desktop/assets/fonts/JetBrainsMono-Regular.woff2`
- `crates/lx-desktop/assets/fonts/JetBrainsMono-Medium.woff2`
- `crates/lx-desktop/assets/fonts/MaterialSymbolsOutlined.woff2`

## Font Path Resolution

Dioxus `asset!()` maps paths relative to the crate root. The CSS file `fonts.css` is loaded via `asset!("/assets/fonts.css")` which means it's served from the Dioxus asset pipeline. Font files placed in `assets/fonts/` are also served by the asset pipeline. The `@font-face` `url()` paths must use absolute paths from the assets root: `url('/assets/fonts/SpaceGrotesk-Regular.woff2')`. This matches how `tailwind.css` is loaded at `/assets/tailwind.css` — the `/assets/` prefix is the asset root in Dioxus's webview serving.

## Steps

### Step 1: Create fonts directory
- Run: `mkdir -p crates/lx-desktop/assets/fonts`

### Step 2: Download font files
- Execute the curl/unzip/rename commands from the "Font Files Required" section above
- Verify all 13 files exist with `ls crates/lx-desktop/assets/fonts/`

### Step 3: Create fonts.css with @font-face declarations
- Create new file: `crates/lx-desktop/assets/fonts.css`
- Content (approximately 95 lines):

```css
@font-face {
  font-family: 'Space Grotesk';
  font-style: normal;
  font-weight: 300;
  font-display: swap;
  src: url('/assets/fonts/SpaceGrotesk-Light.woff2') format('woff2');
}

@font-face {
  font-family: 'Space Grotesk';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/assets/fonts/SpaceGrotesk-Regular.woff2') format('woff2');
}

@font-face {
  font-family: 'Space Grotesk';
  font-style: normal;
  font-weight: 500;
  font-display: swap;
  src: url('/assets/fonts/SpaceGrotesk-Medium.woff2') format('woff2');
}

@font-face {
  font-family: 'Space Grotesk';
  font-style: normal;
  font-weight: 600;
  font-display: swap;
  src: url('/assets/fonts/SpaceGrotesk-SemiBold.woff2') format('woff2');
}

@font-face {
  font-family: 'Space Grotesk';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('/assets/fonts/SpaceGrotesk-Bold.woff2') format('woff2');
}

@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 300;
  font-display: swap;
  src: url('/assets/fonts/Inter-Light.woff2') format('woff2');
}

@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/assets/fonts/Inter-Regular.woff2') format('woff2');
}

@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 500;
  font-display: swap;
  src: url('/assets/fonts/Inter-Medium.woff2') format('woff2');
}

@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 600;
  font-display: swap;
  src: url('/assets/fonts/Inter-SemiBold.woff2') format('woff2');
}

@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 700;
  font-display: swap;
  src: url('/assets/fonts/Inter-Bold.woff2') format('woff2');
}

@font-face {
  font-family: 'JetBrains Mono';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/assets/fonts/JetBrainsMono-Regular.woff2') format('woff2');
}

@font-face {
  font-family: 'JetBrains Mono';
  font-style: normal;
  font-weight: 500;
  font-display: swap;
  src: url('/assets/fonts/JetBrainsMono-Medium.woff2') format('woff2');
}

@font-face {
  font-family: 'Material Symbols Outlined';
  font-style: normal;
  font-weight: 400;
  font-display: block;
  src: url('/assets/fonts/MaterialSymbolsOutlined.woff2') format('woff2');
}

.material-symbols-outlined {
  font-family: 'Material Symbols Outlined';
  font-weight: normal;
  font-style: normal;
  font-size: 24px;
  line-height: 1;
  letter-spacing: normal;
  text-transform: none;
  display: inline-block;
  white-space: nowrap;
  word-wrap: normal;
  direction: ltr;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
  font-feature-settings: 'liga';
  font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
}
```

### Step 4: Register fonts.css as a static asset in app.rs
- Open `crates/lx-desktop/src/app.rs`
- At line 5, find:
```
static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css", AssetOptions::css().with_static_head(true));
```
- Add immediately after (new line 6):
```
static FONTS_CSS: Asset = asset!("/assets/fonts.css", AssetOptions::css().with_static_head(true));
```

### Step 5: Remove Google Fonts CDN link and add fonts.css stylesheet
- In `app.rs`, at lines 18-21, find:
```
    document::Link {
      rel: "stylesheet",
      href: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0&display=swap",
    }
```
- Replace with:
```
    document::Stylesheet { href: FONTS_CSS }
```

### Step 6: Verify tailwind.css font-family references are unchanged
- Confirm `tailwind.css` lines 6-7 still read:
```
  --font-display: 'Space Grotesk', sans-serif;
  --font-body: 'Inter', sans-serif;
```
- No changes needed -- the @font-face declarations in fonts.css make these font-family names resolve to the local files

## File Size Check
- `app.rs`: was 42 lines, now ~41 lines (under 300) -- net change: removed 4 lines of Link, added 1 line of Stylesheet, added 1 static declaration
- `fonts.css`: new file, ~95 lines (under 300)
- `tailwind.css`: unchanged at 196 lines (under 300)

## Verification
- Run `just diagnose` to confirm the build compiles without errors
- Launch the desktop app and verify:
  1. All text renders correctly in Space Grotesk (headings), Inter (body), and JetBrains Mono (code/monospace areas)
  2. All Material Symbols Outlined icons render correctly (check sidebar icons, toolbar icons, etc.)
  3. No network requests to `fonts.googleapis.com` or `fonts.gstatic.com` appear in the webview dev tools
  4. Fonts load without visible FOUT (flash of unstyled text) -- `font-display: swap` for text fonts, `font-display: block` for icon font
