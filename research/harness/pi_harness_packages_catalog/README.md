# Pi Harness Package Catalog

Snapshot of the packages visible on [https://shittycodingagent.ai/packages](https://shittycodingagent.ai/packages) as of 2026-04-19.

The catalog is split into shard files to stay under this repo's 300-line file limit.

## Snapshot

- Visible packages: 1398
- Search results returned by the page query: 1398
- Hidden packages excluded to match page behavior: 0
- Flagged packages still visible: 0
- Packages with declared demo media: 123
- Unique authors: 560
- Packages with at least one type badge: 1368
- Untyped packages: 30
- Multi-type packages: 213

## Type counts

| Type | Count |
| --- | ---: |
| extension | 1287 |
| skill | 240 |
| theme | 46 |
| prompt | 55 |

Type counts are non-exclusive: one package can land in multiple buckets.

## Top 20 by monthly downloads

| Package | Downloads/mo | Types | Author |
| --- | ---: | --- | --- |
| [`pi-subagents`](https://www.npmjs.com/package/pi-subagents) | 29,560 | extension | `nicopreme` |
| [`@a5c-ai/babysitter-pi`](https://www.npmjs.com/package/@a5c-ai/babysitter-pi) | 22,321 | extension, skill | `tmuskal` |
| [`taskplane`](https://www.npmjs.com/package/taskplane) | 21,137 | extension, skill | `henrylach` |
| [`pi-mcp-adapter`](https://www.npmjs.com/package/pi-mcp-adapter) | 18,864 | extension, demo | `nicopreme` |
| [`@ollama/pi-web-search`](https://www.npmjs.com/package/@ollama/pi-web-search) | 15,404 | extension | `jmorgan` |
| [`pi-web-access`](https://www.npmjs.com/package/pi-web-access) | 13,082 | extension, skill, demo | `nicopreme` |
| [`pi-lens`](https://www.npmjs.com/package/pi-lens) | 12,569 | extension, skill | `apmantza` |
| [`pi-markdown-preview`](https://www.npmjs.com/package/pi-markdown-preview) | 11,551 | extension, demo | `omacl` |
| [`@plannotator/pi-extension`](https://www.npmjs.com/package/@plannotator/pi-extension) | 11,350 | extension, skill | `backnotprop` |
| [`pi-gsd`](https://www.npmjs.com/package/pi-gsd) | 8,326 | extension, prompt | `fulgidus` |
| [`@apmantza/greedysearch-pi`](https://www.npmjs.com/package/@apmantza/greedysearch-pi) | 7,163 | extension, skill | `apmantza` |
| [`pi-studio`](https://www.npmjs.com/package/pi-studio) | 6,860 | extension | `omacl` |
| [`@aliou/pi-processes`](https://www.npmjs.com/package/@aliou/pi-processes) | 6,430 | extension, skill, demo | `aliou` |
| [`pi-powerline-footer`](https://www.npmjs.com/package/pi-powerline-footer) | 6,221 | extension | `nicopreme` |
| [`@callumvass/forgeflow-dev`](https://www.npmjs.com/package/@callumvass/forgeflow-dev) | 5,219 | extension, skill | `callumvass` |
| [`pi-docparser`](https://www.npmjs.com/package/pi-docparser) | 4,880 | extension, skill, demo | `maxedapps` |
| [`@companion-ai/feynman`](https://www.npmjs.com/package/@companion-ai/feynman) | 4,792 | extension, skill, prompt | `advaitpaliwal` |
| [`@victor-software-house/pi-openai-proxy`](https://www.npmjs.com/package/@victor-software-house/pi-openai-proxy) | 4,745 | extension | `victor-founder` |
| [`@samfp/pi-memory`](https://www.npmjs.com/package/@samfp/pi-memory) | 4,657 | extension | `samfp` |
| [`@heart-of-gold/toolkit`](https://www.npmjs.com/package/@heart-of-gold/toolkit) | 4,544 | extension, skill | `ondrejsvec` |

## Top authors by package count

| Author | Packages |
| --- | ---: |
| `artale` | 83 |
| `miclivs` | 32 |
| `e9n` | 24 |
| `w-winter` | 19 |
| `moikapy` | 18 |
| `tmustier` | 18 |
| `nicopreme` | 17 |
| `ifiokjr` | 16 |
| `ryan_nookpi` | 15 |
| `hyperprior` | 14 |
| `victor-founder` | 14 |
| `ogulcancelik` | 13 |
| `samfp` | 13 |
| `emiller88` | 12 |
| `siddr` | 12 |

## Catalog shards

- [`packages_01.md`](packages_01.md) — `@0xkobold/pi-alerts` → `@codexstar/pi-pompom` (210 packages)
- [`packages_02.md`](packages_02.md) — `@codexstar/pi-side-chat` → `@justram/pi-undo-redo` (210 packages)
- [`packages_03.md`](packages_03.md) — `@kaiserlich-dev/pi-queue-picker` → `@tianhai/pi-workflow-kit` (210 packages)
- [`packages_04.md`](packages_04.md) — `@tintinweb/pi-subagents` → `pi-claude-cli` (210 packages)
- [`packages_05.md`](packages_05.md) — `pi-claude-oauth-adapter` → `pi-minesweeper` (210 packages)
- [`packages_06.md`](packages_06.md) — `pi-minimax-tools` → `pi-stock-ticker` (210 packages)
- [`packages_07.md`](packages_07.md) — `pi-stories` → `ypi` (138 packages)

## Method

1. Read the rendered packages page and inspected its inline client script.
2. Reused the page's npm search query: `https://registry.npmjs.org/-/v1/search?text=keywords:pi-package`.
3. Pulled each package's `package.json` from jsDelivr with an unpkg fallback to read `pi.extensions`, `pi.skills`, `pi.themes`, `pi.prompts`, `pi.video`, and `pi.image`.
4. Pulled package flag issues from `badlogic/pi-mono` and excluded any package that would be hidden by the page.
5. Sorted the final visible package set A–Z and wrote the catalog into fixed-size markdown shards.
