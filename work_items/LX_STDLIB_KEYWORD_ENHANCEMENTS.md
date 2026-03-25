# Goal

Add missing stdlib operations, fix keyword export prefix, preserve Schema where constraints, and implement agent messaging as runtime builtins.

# Why

- `fs.rm` and `fs.rmdir` don't exist — programs can't delete files/directories in lx.
- Export prefix `+` only works before the keyword (`+Schema Name`), not after (`Schema +Name`). Users expect both to work.
- Schema keyword strips `where` constraints from field declarations. Data contracts with constraints (`confidence: Float where confidence >= 0.0`) can't use Schema.
- Agent trait's `ask` and `tell` methods are stubs returning errors. Agent-to-agent messaging needs runtime implementation.

# What Changes

**fs.rm/rmdir — `crates/lx/src/stdlib/fs.rs`:**

Add two sync builtins:
- `fs.rm(path)` — wraps `std::fs::remove_file`. Returns `Ok ()` on success, `Err` on failure.
- `fs.rmdir(path)` — wraps `std::fs::remove_dir_all`. Returns `Ok ()` on success, `Err` on failure.

Register in `fs.rs` `build()` function.

**Export prefix — `crates/lx/src/parser/stmt_keyword.rs`:**

Currently the keyword parser matches: `keyword_token type_name = body`. The `+` export prefix is handled by the outer `exported` parser in `stmt.rs` which matches `+` BEFORE the keyword token.

To also support `+` after the keyword (before the name), add an optional `Export` token match inside the keyword parser, between the keyword token and the type name:

```rust
keyword
    .then(just(TokenKind::Export).or_not())
    .then(type_name())
    .then_ignore(just(TokenKind::Assign))
    .then(body)
    .map_with(|((((kw, inner_export), name), body), e| {
        let exported = inner_export.is_some();
        KeywordDeclData { keyword: kw, name, exported, ... }
    })
```

The outer `exported` parser also sets `exported`. The final exported flag should be `outer_export || inner_export`. The outer `exported.then(keyword_stmt)` in `stmt.rs` already handles `+Schema Name`. The inner export handles `Schema +Name`. Both produce `exported: true`.

**Schema where constraints — `crates/lx/src/folder/desugar_schema.rs`:**

The Schema desugarer in `desugar_schema.rs` processes `TraitEntry::Field(FieldDecl { name, type_name, default, constraint })`. Currently it reads `default` for the description and ignores `constraint`.

Fix: preserve `constraint` on the generated TraitDecl entries. The `build_validate_method` should also generate constraint checks. For each field with a `where` constraint expression, the validate method should evaluate the constraint and return Err if it fails.

Read `desugar_schema.rs` to see how entries are processed. The `constraint` field is `Option<ExprId>` — an expression that evaluates to Bool. In the generated `validate()`, for each field with a constraint, add a check: if the constraint expression (with the field value substituted) returns false, add the field to the errors.

For `schema()` output, add a `constraints` field to each property in the JSON schema: `{type: "number", description: "...", constraints: "confidence >= 0.0 && confidence <= 1.0"}`. The constraint expression can be rendered as a string by formatting the AST.

**Agent ask/tell — `crates/lx/src/builtins/register.rs` and runtime:**

The Agent trait has `ask = (agent msg) { Err "agent.ask not yet implemented" }` and `tell = (agent msg) { Err "agent.tell not yet implemented" }` in `crates/lx/std/agent.lx`.

Implement agent messaging. The `agent.spawn` builtin already exists as a stub in `register.rs`. The messaging system needs:

1. `agent.spawn(config)` — spawn a child lx process or async task, return an agent handle (an opaque ID or Object). The config has `{command: "lx", args: ["run", "script.lx"]}`.
2. `ask(agent, msg)` — send msg to agent, wait for response. Uses the agent handle to communicate (stdin/stdout JSON, or in-process channel).
3. `tell(agent, msg)` — send msg to agent, don't wait for response (fire-and-forget).

For the initial implementation, use in-process channels (from `std/channel`). `spawn` creates a new async task running the agent script, with a channel pair for communication. `ask` sends on the channel and receives the response. `tell` sends without waiting.

Read `crates/lx/src/builtins/register.rs` `bi_agent_spawn_stub` to see the current stub. Replace with a real implementation. Read `crates/lx/src/stdlib/channel.rs` for channel creation and send/recv patterns to reuse.

Update `crates/lx/std/agent.lx` to replace the error stubs with calls to the builtin agent messaging functions.

# Files Affected

- `crates/lx/src/stdlib/fs.rs` — add rm, rmdir builtins
- `crates/lx/src/parser/stmt_keyword.rs` — accept + after keyword
- `crates/lx/src/folder/desugar_schema.rs` — preserve where constraints
- `crates/lx/src/builtins/register.rs` — implement agent spawn/ask/tell
- `crates/lx/std/agent.lx` — update ask/tell from stubs to builtins

# Task List

### Task 1: Add fs.rm and fs.rmdir

**Subject:** Add file/directory deletion to std/fs

**Description:** Edit `crates/lx/src/stdlib/fs.rs`. Add two sync builtins:

`bi_rm`: takes a path string. Calls `std::fs::remove_file(path)`. Returns `Ok(LxVal::Unit)` on success, `Err(LxError)` with message on failure. If the file doesn't exist, return Ok (idempotent).

`bi_rmdir`: takes a path string. Calls `std::fs::remove_dir_all(path)`. Returns `Ok(LxVal::Unit)` on success, `Err(LxError)` with message on failure. If the directory doesn't exist, return Ok (idempotent).

Register both in the `build()` function: `"rm"/1 => bi_rm`, `"rmdir"/1 => bi_rmdir`.

Write test `tests/fs_rm.lx`:
```lx
use std/fs

fs.mkdir "/tmp/lx_test_rm" ^
fs.write "/tmp/lx_test_rm/file.txt" "hello" ^
assert (fs.exists "/tmp/lx_test_rm/file.txt")

fs.rm "/tmp/lx_test_rm/file.txt" ^
assert (not (fs.exists "/tmp/lx_test_rm/file.txt"))

fs.rmdir "/tmp/lx_test_rm" ^
assert (not (fs.exists "/tmp/lx_test_rm"))
```

**ActiveForm:** Adding fs.rm and fs.rmdir

---

### Task 2: Fix keyword export prefix

**Subject:** Accept + in both positions for keyword declarations

**Description:** Edit `crates/lx/src/parser/stmt_keyword.rs`. The keyword parser currently matches `keyword_token type_name = body`. Add an optional `Export` token between the keyword and the type name.

Read the current keyword_parser function. After matching the keyword kind via `choice(...)`, add `.then(just(TokenKind::Export).or_not())` before `.then(type_name())`. The Export presence sets `exported: true` on the KeywordDeclData.

The outer `exported.then(keyword_stmt)` in `stmt.rs` ALSO sets exported. The final flag should be true if EITHER the outer or inner + is present. In `stmt.rs`, the keyword arm does `d.exported = exp`. Change to `d.exported = exp || d.exported` so the inner export isn't overwritten.

Write test `tests/keyword_export_prefix.lx`:
```lx
+Schema BeforeKeyword = { x: Int = "test" }
Schema +AfterKeyword = { y: Int = "test" }

a = BeforeKeyword {x: 1}
b = AfterKeyword {y: 2}
assert (a.x == 1)
assert (b.y == 2)
```

**ActiveForm:** Fixing keyword export prefix

---

### Task 3: Preserve Schema where constraints

**Subject:** Schema desugarer keeps constraint expressions on TraitDecl fields

**Description:** Edit `crates/lx/src/folder/desugar_schema.rs`. Read the `desugar_schema` function and `build_validate_method`.

Currently, when processing trait entries, the code clears `default` (to extract descriptions) but doesn't clear or preserve `constraint`. Verify: does the generated TraitDecl include the constraint field on each FieldDecl?

If constraints are stripped, preserve them: when building the output `TraitDeclData.entries`, copy the `constraint` field from each input FieldDecl.

For the generated `validate()` method, add constraint checking: for each field with a constraint expression, the validate method should substitute the field value into the constraint expression and check it returns true. This is complex AST generation — for the initial fix, just preserve constraints on the TraitDecl so the type checker can use them. The validate() constraint checking can be a follow-up.

For the generated `schema()` output, if a field has a constraint, include it as a string in the property: `{type: "number", description: "...", constraint: "value >= 0.0"}`. Render the constraint by stringifying the type_name + constraint info.

Write test `tests/schema_where.lx`:
```lx
Schema Bounded = {
  score: Float = "0.0 to 1.0 confidence score"
}

b = Bounded {score: 0.5}
s = b.schema ()
assert (s.properties.score.type == "number")
assert (s.properties.score.description == "0.0 to 1.0 confidence score")
```

**ActiveForm:** Preserving Schema where constraints

---

### Task 4: Implement agent spawn/ask/tell

**Subject:** Replace agent messaging stubs with channel-based implementation

**Description:** Read `crates/lx/src/builtins/register.rs` — find `bi_agent_spawn_stub`, `bi_agent_kill_stub`. These return errors.

Implement `bi_agent_spawn`: takes a config record `{command: "lx", args: ["run", "script.lx"]}`. For in-process agents, spawn an async task that:
1. Reads the script file
2. Creates a channel pair (tx_in/rx_in for sending TO agent, tx_out/rx_out for receiving FROM agent)
3. Creates a child interpreter with `yield` backend that sends on tx_out and receives on rx_in
4. Runs the script in the child interpreter
5. Returns an agent handle record `{id: unique_id, tx: tx_in, rx: rx_out}`

For the initial implementation, use `tokio::spawn` and the channel infrastructure from `std/channel`. The agent handle is a Record with the channel endpoints.

Implement `ask(agent, msg)`: sends msg on agent.tx, receives response on agent.rx. This is an async operation — use `mk_async` for the builtin.

Implement `tell(agent, msg)`: sends msg on agent.tx, doesn't wait. Fire and forget.

Implement `kill(agent)`: drops the agent's channels, causing the spawned task to terminate.

Update `crates/lx/std/agent.lx`:
```lx
ask = (agent_handle msg) { agent.ask agent_handle msg }
tell = (agent_handle msg) { agent.tell agent_handle msg }
```

Where `agent.ask` and `agent.tell` are the builtins registered in `register.rs`.

Write test `tests/agent_spawn.lx`:
```lx
-- This test requires a simple echo agent script
-- For now, just verify the builtins exist and are callable
assert (type_of agent.spawn == "Func")
assert (type_of agent.kill == "Func")
```

A full integration test with actual agent spawning is complex and depends on the yield backend. Start with verifying the builtins exist and handle basic config validation.

**ActiveForm:** Implementing agent messaging

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_STDLIB_KEYWORD_ENHANCEMENTS.md" })
```
