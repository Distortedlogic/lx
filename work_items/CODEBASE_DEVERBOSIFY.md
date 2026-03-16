# Codebase Deverbosification: Remove Unnecessary Verbosity Across lx Implementation

## Goal

Systematically eliminate redundant code, duplicated logic, non-idiomatic patterns, and unnecessary allocations across the entire lx Rust implementation. The codebase has 25,000 lines across 100+ files. This work item targets 29 concrete improvements that reduce line count, improve idiom compliance, and bring all files under the CLAUDE.md 300-line limit.

## Why

- 161 sites construct `Value::Record(Arc::new(...))` from manually built `IndexMap` across 61 files (141 in stdlib across 53 files, plus 20 in builtins, interpreter, and backends across 8 files) — nearly every module pays this boilerplate tax
- 57 sites across 22 stdlib files use `.get("key").and_then(|v| v.as_str())` chains when a single method on `Value` would eliminate the noise
- 13 `let _ =` sites swallow `Result` values across 9 files, violating the CLAUDE.md no-swallowed-errors rule and hiding real failures
- 24 files exceed the 300-line limit (one is 812 lines)
- 42 parser sites clone `TokenKind` (which contains heap `String` variants) just to match on it, across 6 parser files
- The parser lacks `expect_ident()` / `expect_type_name()` helpers, causing the same 5-8 line extraction pattern to repeat ~14 times
- Multiple interpreter, builtin, lexer, and stdlib modules contain near-identical logic blocks that differ by one parameter

## What Changes

**Foundation helpers (Tasks 1-2):** Add a `record!` macro to `value.rs` that builds `Value::Record(Arc::new(indexmap!{...}))` from key-value pairs. Add `str_field`, `int_field`, `float_field`, `bool_field` convenience methods on `Value` that combine `.get()` + `.and_then()` in one call.

**Core type fixes (Task 3):** Replace 5 `Option::None` with `None` and 3 `sort_by_key` clone-per-comparison calls with `sort_by` reference comparison in `value.rs`. (Note: the `hash_value` List/Tuple merge is already done.)

**Parser helpers (Tasks 4-7):** Add `expect_ident()` and `expect_type_name()` to the parser, then apply them across all parser files. Eliminate `peek().clone()` by matching on references. Extract `parse_binary_infix` helper in `infix.rs`.

**Lexer cleanup (Task 8):** Extract `collect_digits_no_underscores` in `numbers.rs` and `flush_buf` for the 4 mid-string `mem::take` sites in `strings.rs` (the remaining end-of-function sites move `buf` by value and require no change).

**Builtin deduplication (Tasks 9-10):** Extract `str_transform` helper for 5 identical string functions and unify `pad_left`/`pad_right` in `str.rs`. Unify 4 identical log functions into a single `make_log_fn` parameterized by level in `mod.rs`.

**Interpreter deduplication (Tasks 11-14):** Extract `call_in_env` and unify `make_section_func`/`make_section_func_2` in `apply.rs`. Extract `extract_pid` in `agents.rs`. Consolidate 5 identical export arms in `modules.rs` (Binding and TypeDef excluded — both have distinct logic). Extract `eval_short_circuit` for And/Or in `eval.rs`.

**Error swallowing (Task 15):** Fix all 13 `let _ =` error-swallowing sites across 9 files — propagate with `?`, log, or return explicit `Err`.

**Stdlib agent deduplication (Tasks 16-18):** Consolidate 3 restart functions in `agent_supervise.rs`. Extract shared publish iteration in `agent_pubsub.rs`. Extract record-building factory in `agents_reviewer.rs`.

**Backends and CLI cleanup (Tasks 19-20):** Extract error-field builder in `defaults.rs`. Extract `exit_err`, move mid-function imports in `main.rs`.

**Checker and misc fixes (Tasks 21-23):** Rename unused synth result bindings to `let _ =` in `synth.rs` (keep all calls — they have side effects on the unification table). Use `ToPrimitive::to_f64()` / `to_usize()` instead of string round-trips in `hof.rs` and `budget.rs`, fix full-path `indexmap::IndexMap` in `hof_extra.rs`. Change `HashMap<String, bool>` to `HashSet<String>` in `agent_reconcile_strat.rs`, fix fragile epsilon comparison.

**Bulk application (Tasks 24-25):** Apply `record!` macro across all 61 files (stdlib, builtins, interpreter, backends) with manual IndexMap record construction. Apply field accessor helpers across all 22 stdlib files with `.get().and_then()` chains.

**File splits (Tasks 26-29):** Split files still exceeding 300 lines after prior tasks, grouped by area: parser, interpreter, stdlib, core/CLI. Re-check line counts before each split — skip any file that prior tasks already brought under 300.

## How It Works

Tasks are ordered so each leaves the code in a compilable state. Foundation helpers (Tasks 1-2) are additive — they introduce new APIs without changing existing code. Core type fix (Task 3) is localized to `value.rs`. Parser and lexer refactors (Tasks 4-8) are self-contained within their directories. Builtin and interpreter refactors (Tasks 9-14) restructure internal logic without changing external signatures. Error swallowing fixes (Task 15) come after structural refactors so file layouts are stable. Stdlib agent dedup (Tasks 16-18) and backend/CLI cleanup (Tasks 19-20) are independent of each other. Bulk application tasks (24-25) come after all helpers exist and after structural refactors are done. File splits (26-29) come last because prior tasks reduce file sizes — re-check each file's line count before splitting and skip files already under 300.

## Files Affected

**Core:** `value.rs`, `ast.rs`

**Lexer:** `lexer/mod.rs`, `lexer/numbers.rs`, `lexer/strings.rs`

**Parser:** `parser/mod.rs`, `parser/statements.rs`, `parser/prefix.rs`, `parser/infix.rs`, `parser/paren.rs`, `parser/pattern.rs`, `parser/func.rs`, `parser/type_ann.rs`

**Interpreter:** `interpreter/mod.rs`, `interpreter/eval.rs`, `interpreter/apply.rs`, `interpreter/agents.rs`, `interpreter/modules.rs`, `interpreter/patterns.rs`, `interpreter/refine.rs`, `interpreter/collections.rs`

**Builtins:** `builtins/mod.rs`, `builtins/str.rs`, `builtins/coll.rs`, `builtins/hof.rs`, `builtins/hof_extra.rs`

**Checker:** `checker/synth.rs`

**Backends:** `backends/defaults.rs`, `backends/user.rs`

**CLI:** `lx-cli/src/main.rs`

**Stdlib (all 66):** `agent.rs`, `agent_capability.rs`, `agent_dialogue.rs`, `agent_dispatch.rs`, `agent_gate.rs`, `agent_handoff.rs`, `agent_intercept.rs`, `agent_mock.rs`, `agent_negotiate.rs`, `agent_pubsub.rs`, `agent_reconcile.rs`, `agent_reconcile_strat.rs`, `agent_supervise.rs`, `agents_auditor.rs`, `agents_grader.rs`, `agents_monitor.rs`, `agents_planner.rs`, `agents_reviewer.rs`, `agents_router.rs`, `ai.rs`, `ai_structured.rs`, `audit.rs`, `budget.rs`, `circuit.rs`, `context.rs`, `cron.rs`, `ctx.rs`, `diag.rs`, `diag_walk.rs`, `env.rs`, `fs.rs`, `git.rs`, `git_branch.rs`, `git_diff.rs`, `git_diff_parse.rs`, `git_log.rs`, `git_ops.rs`, `git_status.rs`, `http.rs`, `introspect.rs`, `json.rs`, `json_conv.rs`, `knowledge.rs`, `math.rs`, `mcp.rs`, `mcp_http.rs`, `mcp_rpc.rs`, `mcp_stdio.rs`, `md.rs`, `md_build.rs`, `memory.rs`, `plan.rs`, `pool.rs`, `profile.rs`, `profile_io.rs`, `profile_strategy.rs`, `prompt.rs`, `re.rs`, `retry.rs`, `saga.rs`, `tasks.rs`, `time.rs`, `trace.rs`, `trace_progress.rs`, `trace_query.rs`, `user.rs`

## Task List

### Task 1: Add record! macro to value.rs

**Subject:** Create a record! macro that builds Value::Record from key-value pairs

**Description:** In `crates/lx/src/value.rs`, add a `macro_rules! record!` that accepts `{ "key" => expr, ... }` syntax and expands to `Value::Record(Arc::new({ let mut m = IndexMap::new(); m.insert("key".into(), expr); ... m }))`. Export the macro from the crate root in `lib.rs`. The macro must handle zero or more pairs and use `String` keys (matching `Record(Arc<IndexMap<String, Value>>)`). Verify it compiles by running `just diagnose`.

**ActiveForm:** Adding record! builder macro

---

### Task 2: Add field accessor methods to Value

**Subject:** Add str_field, int_field, float_field, bool_field methods on Value

**Description:** In `crates/lx/src/value.rs`, add these methods to the `impl Value` block:
- `fn str_field(&self, key: &str) -> Option<&str>` — if self is Record, get key and call as_str
- `fn int_field(&self, key: &str) -> Option<&num_bigint::BigInt>` — if self is Record, get key and extract Int ref
- `fn float_field(&self, key: &str) -> Option<f64>` — if self is Record, get key and extract Float
- `fn bool_field(&self, key: &str) -> Option<bool>` — if self is Record, get key and extract Bool
- `fn list_field(&self, key: &str) -> Option<&[Value]>` — if self is Record, get key and extract List slice
- `fn record_field(&self, key: &str) -> Option<&IndexMap<String, Value>>` — if self is Record, get key and extract Record fields

Each method returns None if self is not a Record, the key is missing, or the value is the wrong variant. After adding the Value methods, delete the existing local free-function duplicates: `str_field` in `stdlib/ai.rs`, `int_field` and `float_field` in `stdlib/circuit.rs`, and migrate their callers to the new Value methods. Run `just diagnose` to verify.

**ActiveForm:** Adding field accessor methods to Value

---

### Task 3: Fix value.rs idiom issues

**Subject:** Replace Option::None with None, fix sort_by_key cloning

**Description:** In `crates/lx/src/value.rs`:
- Replace all 5 instances of `Option::None` with `None` in the `as_int`, `as_float`, `as_str`, `as_bool`, `as_list` methods
- Replace all 3 instances of `sort_by_key(|(k, _)| (*k).clone())` with `sort_by(|a, b| a.0.cmp(&b.0))` — this eliminates a String allocation per comparison during sorting

(Note: the `hash_value` List/Tuple or-pattern merge is already done at line 174.)

Run `just diagnose` to verify.

**ActiveForm:** Fixing value.rs idiom issues

---

### Task 4: Add expect_ident and expect_type_name parser helpers

**Subject:** Add helper methods to Parser for extracting ident and type name tokens

**Description:** In `crates/lx/src/parser/mod.rs`, add two methods to the Parser impl:
- `fn expect_ident(&mut self, context: &str) -> Result<String, LxError>` — checks if current peek is `TokenKind::Ident(name)`, advances and returns the name, otherwise returns a parse error using context for the message (e.g. "expected identifier in {context}")
- `fn expect_type_name(&mut self, context: &str) -> Result<String, LxError>` — same but for `TokenKind::TypeName(name)`

Both methods must capture the span from the current token for error reporting. Run `just diagnose` to verify.

**ActiveForm:** Adding expect_ident and expect_type_name parser helpers

---

### Task 5: Apply parser helpers across parser files

**Subject:** Replace manual ident/type-name extraction with expect_ident/expect_type_name

**Description:** Across all parser files (`statements.rs`, `prefix.rs`, `pattern.rs`, `func.rs`, `type_ann.rs`, `paren.rs`), find every instance of the pattern: `match self.peek().clone() { TokenKind::Ident(n) => { self.advance(); n } _ => return Err(...) }` (and the TypeName equivalent) and replace with `self.expect_ident("context")?` or `self.expect_type_name("context")?`. The context string should describe what was being parsed (e.g. "binding name", "protocol field", "trait name"). Run `just diagnose` and `just test` to verify no behavior change.

**ActiveForm:** Applying parser helpers across parser files

---

### Task 6: Eliminate peek().clone() across parser files

**Subject:** Match on references from peek() instead of cloning TokenKind

**Description:** Across all parser files, find instances of `match self.peek().clone() { ... }` where the matched value is only used for discriminant checking (not for extracting owned data). Replace with `match self.peek() { ... }` matching on references. For arms that need owned data (like extracting a String from `Ident(name)`), keep the clone only for that specific arm by calling `self.advance()` first then extracting from the consumed token. The goal is to eliminate unnecessary String heap allocations during parsing. There are 42 such sites across `statements.rs` (29), `prefix.rs` (5), `type_ann.rs` (3), `pattern.rs` (3), `paren.rs` (1), `mod.rs` (1). Run `just diagnose` and `just test` to verify.

**ActiveForm:** Eliminating unnecessary peek().clone() calls

---

### Task 7: Extract parse_binary_infix in parser/infix.rs

**Subject:** Deduplicate Pipe, TildeArrow, TildeArrowQ, QQ arms into shared helper

**Description:** In `crates/lx/src/parser/infix.rs`, the match arms for `Pipe`, `TildeArrow`, `TildeArrowQ`, and `QQ` all follow the same structure: parse right operand with given binding power, construct a binary expression with two boxed sub-expressions. Extract a helper method `fn parse_binary_infix(&mut self, left: SExpr, rbp: u8, make: fn(Box<SExpr>, Box<SExpr>) -> Expr, start: u32) -> Result<SExpr, LxError>` and call it from each arm. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Extracting parse_binary_infix helper

---

### Task 8: Extract lexer helpers in numbers.rs and strings.rs

**Subject:** Deduplicate digit collection and buffer flush patterns in lexer

**Description:** In `crates/lx/src/lexer/numbers.rs`:
- Extract the duplicated `self.source[start..self.pos].chars().filter(|c| *c != '_').collect::<String>()` (appears twice) into a method `fn collect_digits(&self, start: usize) -> String`

In `crates/lx/src/lexer/strings.rs`:
- Extract the repeated mid-string `if !buf.is_empty() { parts.push(Token::new(TokenKind::..., ..., span)); }` pattern (the 4 sites that use `mem::take(&mut buf)`) into a closure or method `flush_buf`. The remaining end-of-function sites move `buf` by value and require no change

Run `just diagnose` to verify.

**ActiveForm:** Extracting lexer helpers

---

### Task 9: Deduplicate string builtins in builtins/str.rs

**Subject:** Extract str_transform helper for trim/case functions, unify pad functions

**Description:** In `crates/lx/src/builtins/str.rs`:
- Create a helper function `fn str_transform(args: &[Value], span: Span, name: &str, f: fn(&str) -> String) -> Result<Value, LxError>` that matches `args[0]` as `Value::Str`, applies `f`, wraps in `Value::Str(Arc::from(...))`, or returns a type error
- Rewrite `bi_upper`, `bi_lower`, `bi_trim`, `bi_trim_start`, `bi_trim_end` to each be a one-line call to `str_transform` with the appropriate function pointer (`str::to_uppercase`, `str::to_lowercase`, `str::trim` + `.to_string()`, etc.)
- Unify `bi_pad_left` and `bi_pad_right` into a single helper parameterized by alignment direction, called from two thin wrapper functions

Run `just diagnose` and `just test` to verify.

**ActiveForm:** Deduplicating string builtins

---

### Task 10: Unify log functions in builtins/mod.rs

**Subject:** Replace 4 near-identical log functions with a single parameterized factory

**Description:** In `crates/lx/src/builtins/mod.rs`, the `make_log_builtin` function is called four times to create `log_info`, `log_warn`, `log_err`, `log_debug` — each closure body is structurally identical, differing only in which `LogBackend` method is called. Refactor so that a single closure factory takes a log-level discriminant (use a simple enum or string) and dispatches to the correct backend method internally. The four builtin registrations should each be one line calling the factory. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Unifying log functions

---

### Task 11: Deduplicate interpreter/apply.rs

**Subject:** Extract call_in_env helper and unify section function constructors

**Description:** In `crates/lx/src/interpreter/apply.rs`:
- The function call environment setup (save env, create child, bind params, eval body, restore env, handle ControlFlow) appears twice with identical logic. Extract into a private method `fn call_in_env(&mut self, params: &[Param], args: &[Value], body: &[Stmt], span: Span) -> Result<Value, LxError>`
- `make_section_func` and `make_section_func_2` are identical except for parameter count and names. Unify into a single function that takes a parameter name list
- The index access logic for Tuple and List (both compute index identically and use `.get(i).cloned().ok_or_else(...)`) should be extracted into a shared helper

Run `just diagnose` and `just test` to verify.

**ActiveForm:** Deduplicating interpreter/apply.rs

---

### Task 12: Extract extract_pid in interpreter/agents.rs

**Subject:** Deduplicate PID extraction from agent records

**Description:** In `crates/lx/src/interpreter/agents.rs`, both `eval_agent_send` and `eval_agent_ask` extract `__pid` from a record, convert to u32, and produce the same error. Extract a private function `fn extract_pid(agent: &Value, span: Span) -> Result<u32, LxError>` and call it from both sites. Also deduplicate the field deduplication pattern in `eval_protocol_def` where both spread entries and regular field entries use identical `.position()` then update-or-push logic — extract a helper. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Extracting PID extraction helper

---

### Task 13: Consolidate export-collection arms in interpreter/modules.rs

**Subject:** Replace 5 identical match arms with a single export-handling block

**Description:** In `crates/lx/src/interpreter/modules.rs`, the `collect_exports` function has five match arms (`Protocol`, `ProtocolUnion`, `McpDecl`, `TraitDecl`, `AgentDecl`) that all do: `if exported { extract name from variant; if let Some(val) = env.get(&name) { bindings.insert(name, val.clone()) } }`. Extract the name and exported flag from each variant into a tuple first (e.g. via a helper method on Stmt or a local closure), then handle the common logic once. Keep `Binding` and `TypeDef` as their own arms — `Binding` has distinct pattern-matching and target-name extraction logic, and `TypeDef` additionally manages `variant_ctors`. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Consolidating export-collection arms

---

### Task 14: Extract eval_short_circuit in interpreter/eval.rs

**Subject:** Deduplicate And/Or operator evaluation

**Description:** In `crates/lx/src/interpreter/eval.rs`, the `And` and `Or` operator handling (eval left, force_defaults, as_bool, conditional return) follows an identical structure differing only in the short-circuit condition. Extract a private method `fn eval_short_circuit(&mut self, left: &SExpr, right: &SExpr, is_and: bool, span: Span) -> Result<Value, LxError>` and call it from both arms. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Extracting eval_short_circuit helper

---

### Task 15: Fix error swallowing across all files

**Subject:** Replace all `let _ =` on Result values with explicit handling

**Description:** Fix all 13 `let _ =` error-swallowing sites across 9 files. For each site, apply the appropriate fix:
- **agent.rs** (2 sites: child.kill, child.wait): Log the error via `eprintln!` since these are cleanup operations
- **agent_supervise.rs** (2 sites: child.kill, child.wait): Same as agent.rs — log cleanup errors
- **agent_pubsub.rs** (1 site: ask_agent): Collect errors and return them so callers know about delivery failures
- **saga.rs** (1 site: call_value compensation): Log the error since compensation is best-effort
- **backends/user.rs** (3 sites: stderr flush x2, remove_file): Log via `eprintln!`
- **main.rs** (1 site: stdout flush): Use `?` since main returns ExitCode or propagate
- **mcp_http.rs** (1 site: client.delete.send): Propagate with `?` or log
- **checker/synth.rs** (1 site: table.unify): Propagate with `?`
- **interpreter/eval.rs** (1 site: call_value close_fn): Log the error since this is resource cleanup

Note: the remaining `let _ =` sites in the codebase (26 total in git_branch.rs, git.rs, git_status.rs, env.rs, cron.rs, time.rs, md.rs, interpreter/mod.rs, interpreter/eval.rs, checker/mod.rs) are benign — they are parameter bindings (`let _ = &args[0]`), variable discards (`let _ = name`, `let _ = exported`, `let _ = span`), intentional Option discards (`let _ = stack.pop()`), or intentional Type value discards (`let _ = named_to_type(...)`, `let _ = resolve_mcp_output(...)` where the call has side effects), not error swallowing.

Run `just diagnose` and `just test` to verify no behavioral regressions.

**ActiveForm:** Fixing error swallowing

---

### Task 16: Consolidate restart functions in agent_supervise.rs

**Subject:** Unify restart_child, restart_all, restart_from into single helper

**Description:** In `crates/lx/src/stdlib/agent_supervise.rs`, the three restart functions (`restart_child`, `restart_all`, `restart_from`) duplicate restart logic differing only by which children are restarted. Extract a single `fn restart_range(children: &mut [Child], range: Range<usize>, span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>` and call it from the three strategy handlers with the appropriate range. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Consolidating restart functions

---

### Task 17: Extract shared publish iteration in agent_pubsub.rs

**Subject:** Deduplicate topic lookup and subscriber iteration

**Description:** In `crates/lx/src/stdlib/agent_pubsub.rs`, both `bi_publish` and `bi_publish_collect` repeat the `TOPICS.get(&topic)` lookup and subscriber iteration loop. Extract a shared function `fn publish_to_subscribers(topic: &str, msg: &Value, span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Vec<Value>, LxError>` and call it from both. `bi_publish` ignores the results, `bi_publish_collect` returns them. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Extracting shared publish iteration

---

### Task 18: Extract record-building factory in agents_reviewer.rs

**Subject:** Deduplicate record construction for mistakes and facts

**Description:** In `crates/lx/src/stdlib/agents_reviewer.rs`, the record-building code for mistakes and facts uses nearly identical logic. Extract a helper function that takes the category label and list of items and returns a `Value::Record`. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Extracting record-building factory

---

### Task 19: Extract error-field builder in backends/defaults.rs

**Subject:** Deduplicate HTTP error response record construction

**Description:** In `crates/lx/src/backends/defaults.rs`, apply the `record!` macro from Task 1 to the `response_to_value` builder and any other `IndexMap` record construction sites. Also fix the 3 instances of `Arc::from(OWNED_STRING.as_str())` (lines 51, 67, 166) to `Arc::from(OWNED_STRING)` — the `.as_str()` is an unnecessary indirection since `Arc::from` accepts `String` directly. Run `just diagnose` to verify.

**ActiveForm:** Extracting error-field builder

---

### Task 20: Fix CLI main.rs issues

**Subject:** Extract exit_err helper, move mid-function imports to file level

**Description:** In `crates/lx-cli/src/main.rs`:
- Move `use std::io::BufRead` and `use std::io::Write` from mid-function positions to the import block at the top of the file
- The `Err(e) => { eprintln!("...error: {e}"); return ExitCode::from(1); }` pattern appears 8 times (across `run_file`, `run_tests`, `run_agent`, `run_diagram`, and `read_and_parse`) — extract a helper function `fn exit_with_error(prefix: &str, e: impl std::fmt::Display) -> ExitCode`
- The `NamedSource` + `miette::Report` error display pattern appears 6 times — extract a helper

Run `just diagnose` to verify.

**ActiveForm:** Fixing CLI main.rs issues

---

### Task 21: Fix checker/synth.rs unused synth results

**Subject:** Rename unused synth result bindings

**Description:** In `crates/lx/src/checker/synth.rs`, there are 4 instances of unused `let _<name> = self.synth(...)` bindings (specifically `_arg_t`, `_at`, `_et`, `_st`) where the result is assigned to a variable prefixed with `_` and never used. Note that `synth()` returns a `Type`, not a `Result` — these are not error-swallowing sites. The `synth()` calls have side effects on the unification table (registering type bindings) and must be kept unconditionally. Rename each to `let _ =` to clearly signal intentional discard. Do not remove any calls. Run `just diagnose` to verify.

**ActiveForm:** Fixing unused synth result bindings

---

### Task 22: Fix ToPrimitive usage and import path

**Subject:** Use ToPrimitive instead of string round-trips, fix IndexMap import

**Description:**
- In `crates/lx/src/builtins/hof.rs`: Replace both instances of `usize::try_from(n.clone()).unwrap_or(0)` (in `bi_take` at line 185 and `bi_drop` at line 196) with `n.to_usize().ok_or_else(|| LxError::runtime("...", span))?` using the `ToPrimitive` trait from `num-traits` (already a transitive dependency via `num-bigint`). Add `use num_traits::ToPrimitive;` at the top. This eliminates a BigInt clone and replaces a silent default-to-0 with an explicit error.
- In `crates/lx/src/stdlib/budget.rs`: Replace all 4 instances of string-roundtrip numeric conversion (lines 66, 90, 98, 245 — only line 66 uses the explicit `::<f64>` turbofish, the others infer the type) with `n.to_f64().ok_or_else(...)` using ToPrimitive.
- In `crates/lx/src/builtins/hof_extra.rs`: Replace the full path `indexmap::IndexMap::new()` with `IndexMap::new()` and add `use indexmap::IndexMap;` to the imports.

Run `just diagnose` to verify.

**ActiveForm:** Fixing ToPrimitive usage and import path

---

### Task 23: Fix agent_reconcile_strat.rs type and comparison issues

**Subject:** Change HashMap bool tracker to HashSet, fix epsilon comparison

**Description:** In `crates/lx/src/stdlib/agent_reconcile_strat.rs`:
- Replace `HashMap<String, bool>` used as `seen_in_result` with `HashSet<String>` — the bool value is never read, only insertion is checked
- Replace `(winner_weight - total_weight).abs() < f64::EPSILON` with `(winner_weight - total_weight).abs() < 1e-10` — `f64::EPSILON` is the smallest representable difference near 1.0, which is far too tight for accumulated floating-point sums; `1e-10` is a practical tolerance for weight comparisons

Run `just diagnose` and `just test` to verify.

**ActiveForm:** Fixing reconcile strategy types

---

### Task 24: Apply record! macro across all files

**Subject:** Replace manual IndexMap construction with record! macro in all modules

**Description:** Across all files that have the pattern `let mut f = IndexMap::new(); f.insert("key".into(), value); ... Value::Record(Arc::new(f))` (161 occurrences across 61 files — 141 in stdlib across 53 files, plus 20 in builtins, interpreter, and backends across 8 files), replace with the `record!` macro from Task 1. Use `rg 'Value::Record\(Arc::new\(' crates/lx/src/ -l` to discover all target files rather than relying on a fixed list — the pattern is pervasive across nearly all modules. Run `just diagnose` and `just test` after completing all replacements.

**ActiveForm:** Applying record! macro across all modules

---

### Task 25: Apply field accessor helpers across stdlib files

**Subject:** Replace .get().and_then() chains with field accessor methods in all stdlib modules

**Description:** Across all stdlib files that use `.get("key").and_then(|v| v.as_str())` (57 occurrences across 22 stdlib files) and similar patterns for int/float/bool, replace with the `str_field`, `int_field`, `float_field`, `bool_field` methods from Task 2. Use `rg '\.get\(.*\)\.and_then\(\|v\| v\.as_' crates/lx/src/stdlib/ -l` to discover all target files. For non-Record values, the caller typically destructures first — in those cases, adjust the call pattern to use the Value methods. Run `just diagnose` and `just test` after completing all replacements.

**ActiveForm:** Applying field accessor helpers across stdlib

---

### Task 26: Split over-limit parser files

**Subject:** Split statements.rs, prefix.rs, mod.rs, and infix.rs under 300-line limit

**Description:**
- `parser/statements.rs` (812 lines): Split into `statements.rs` (core: bindings, functions, type defs, for/while), `stmt_protocol.rs` (Protocol, ProtocolUnion, protocol field parsing), `stmt_mcp.rs` (MCP declarations), `stmt_agent.rs` (Agent declarations, Trait declarations). Each new file contains the relevant parsing functions and is called from `statements.rs` via methods on the Parser. Re-export or make pub(crate) as needed.
- `parser/prefix.rs` (469 lines): Extract collection/container literal parsing into `parser/prefix_coll.rs`. Keep core prefix dispatch in prefix.rs.
- `parser/mod.rs` (411 lines): Extract error recovery and utility methods into `parser/helpers.rs`. Keep the main Parser struct, new(), and top-level parse entry points in mod.rs.
- `parser/infix.rs` (308 lines): Re-check line count first — if prior tasks brought it under 300, skip. Otherwise extract slice parsing logic into the existing `paren.rs` or a new `parser/slice.rs`, bringing infix.rs under 300.

Run `just diagnose` and `just test` to verify.

**ActiveForm:** Splitting over-limit parser files

---

### Task 27: Split over-limit interpreter files

**Subject:** Split mod.rs, eval.rs, agents.rs, apply.rs under 300-line limit

**Description:**
- `interpreter/mod.rs` (475 lines): Extract statement execution into `interpreter/exec_stmt.rs` — the large match on Stmt variants. Keep struct definition, new(), with_env(), and the top-level run() in mod.rs.
- `interpreter/eval.rs` (472 lines): Extract literal evaluation and binary operator handling into `interpreter/eval_ops.rs`. Keep the main eval() dispatch in eval.rs.
- `interpreter/agents.rs` (467 lines): Extract MCP declaration evaluation and protocol constraint validation into `interpreter/agents_mcp.rs` or `interpreter/agents_protocol.rs`. Keep send/ask/spawn in agents.rs.
- `interpreter/apply.rs` (425 lines): Extract section-function construction and index access into `interpreter/apply_helpers.rs`. Keep the main apply logic in apply.rs.

All new files contain private functions called from the original. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Splitting over-limit interpreter files

---

### Task 28: Split over-limit stdlib files

**Subject:** Split 10 over-limit stdlib files under 300-line limit

**Description:** Split each of these files by extracting logical sub-sections into new sibling files:
- `memory.rs` (417) → extract storage/persistence functions into `memory_store.rs`
- `tasks.rs` (379) → extract task query/filter functions into `tasks_query.rs`
- `context.rs` (375) → extract eviction logic into `context_evict.rs`
- `diag_walk.rs` (355) → extract expression walking into `diag_walk_expr.rs`
- `audit.rs` (350) → extract scoring/analysis into `audit_score.rs`
- `budget.rs` (344) → extract projection/reporting into `budget_report.rs`
- `prompt.rs` (332) → extract rendering into `prompt_render.rs`
- `agents_grader.rs` (324) → extract rubric evaluation into `agents_grader_rubric.rs`
- `md.rs` (314) → re-check line count first; if still over 300, extract section/list building into `md_parse.rs` (distinct from existing `md_build.rs`)
- `mcp.rs` (304) → re-check line count first; if still over 300, extract tool generation into `mcp_tools.rs`

Update `stdlib/mod.rs` to include new submodules. Run `just diagnose` and `just test` to verify.

**ActiveForm:** Splitting over-limit stdlib files

---

### Task 29: Split over-limit core files

**Subject:** Split ast.rs, value.rs, lexer/mod.rs, builtins/coll.rs, builtins/mod.rs, main.rs under 300-line limit

**Description:**
- `ast.rs` (500 lines): Extract Display impls into `ast_display.rs`. Keep struct/enum definitions in ast.rs.
- `value.rs` (318 lines): Re-check line count first; if still over 300, extract PartialEq, Hash, and helper impls into `value_impls.rs`. Keep enum definition and core methods in value.rs.
- `lexer/mod.rs` (353 lines): Extract keyword/operator token matching into `lexer/keywords.rs`. Keep the main next_token loop in mod.rs.
- `builtins/coll.rs` (406 lines): Extract sort/group/transform functions into `builtins/coll_transform.rs`. Keep core access/query functions in coll.rs.
- `builtins/mod.rs` (370 lines): Extract builtin registration (the large function that inserts all builtins into the env) into `builtins/register.rs`. Keep individual builtin implementations in mod.rs.
- `lx-cli/src/main.rs` (303 lines): Re-check line count first; if still over 300, extract the test runner logic into a separate function or module in the same crate to bring main.rs under 300.

Run `just diagnose` and `just test` to verify.

**ActiveForm:** Splitting over-limit core files

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written. Exception: file-split tasks (26-29) with a "re-check line count first" gate may be skipped for individual files already under 300 lines.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/CODEBASE_DEVERBOSIFY.md" })
```

Then call `next_task` to begin. After completing each task's implementation, call `complete_task` to format, commit, and run diagnostics. Repeat until all tasks are done.
