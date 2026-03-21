Done. Here's what I did:

1. **Fixed `pkg/data/vectors.lx`** — `cosine_sim` and `add_batch` used `pair.[0]`/`pair.[1]` on zip tuples, which doesn't work. Changed to tuple destructuring `(ai bi) = pair` and `(item vec) = pair`.

2. **Updated `tests/107_vectors.lx`** — Added tests for embed-dependent methods (`add`, `add_batch`, `search`) that gracefully skip when `VOYAGE_API_KEY` is not set, following the same `Err e -> log.info "skipped"` pattern used by `101_embed.lx`.

The test covers:
- `add_raw`/`search_raw` (always run, no embed needed)
- Collection methods: `get`, `remove`, `len`, `save`, `load`
- `add`, `add_batch`, `search` (skip gracefully without embed provider)