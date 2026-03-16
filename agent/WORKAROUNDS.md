# Workarounds

Temporary limitations from incomplete implementation. Each entry should go away when the relevant feature lands.

## Lexer

- **Unicode in comments causes panics.** The lexer uses byte indexing, so multi-byte chars (em-dash, smart quotes, etc.) in comments crash at `lexer/mod.rs:88`. **Workaround:** Avoid non-ASCII in `.lx` files. **Fix:** Switch lexer to char-based indexing.

## RuntimeCtx

- **`UserBackend` default is `NoopUserBackend`, not `StdinStdoutUserBackend`.** The CLI doesn't yet upgrade to `StdinStdoutUserBackend` for interactive terminal use. **Workaround:** `std/user` interactive functions (confirm, choose, ask) auto-approve/pick-first/return-empty in all contexts including `lx run`. **Fix:** CLI detects TTY and sets `StdinStdoutUserBackend`.

## Value::Agent

- **`uses` and `on` fields are parsed but not stored in the runtime value.** The AST captures them, but `Value::Agent` only holds `name`, `traits`, `methods`, and `init`. MCP auto-connect (`uses`) and lifecycle hooks (`on`) are not wired up yet. **Workaround:** Manage MCP connections and hooks manually in method bodies. **Fix:** Implement `uses` MCP lifecycle (connect on first call, close on drop) and `on` hook registration.
