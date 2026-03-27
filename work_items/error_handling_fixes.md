# Goal

Fix six locations where errors are silently swallowed or inappropriate defaults mask failures. Each change either propagates the error, logs it, or replaces `unwrap_or_default()` with `expect()` where failure is impossible.

# Files

- `/home/entropybender/repos/lx/crates/lx/src/stdlib/store/store_dispatch.rs`
- `/home/entropybender/repos/lx/crates/lx/src/interpreter/apply.rs`
- `/home/entropybender/repos/lx/crates/lx-desktop/src/terminal/view.rs`
- `/home/entropybender/repos/lx/crates/lx-mobile/src/pages/approvals.rs`
- `/home/entropybender/repos/lx/crates/lx-cli/src/agent_cmd.rs`
- `/home/entropybender/repos/lx/crates/lx/src/builtins/agent.rs`

# Steps

## Step 1: Propagate serialization error in store save path

**File:** `/home/entropybender/repos/lx/crates/lx/src/stdlib/store/store_dispatch.rs`

Find and replace the following exact string:
```
  let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();
```
With:
```
  let pretty = serde_json::to_string_pretty(&json_val).map_err(|e| LxError::runtime(format!("store.save: serialization failed: {e}"), span))?;
```

## Step 2: Return Result from store_clone on missing store

This requires changing the function signature and updating its one call site.

### Step 2a: Change store_clone signature

**File:** `/home/entropybender/repos/lx/crates/lx/src/stdlib/store/store_dispatch.rs`

Find and replace the following exact string:
```
pub fn store_clone(id: u64) -> u64 {
  let data = STORES.get(&id).map(|s| s.data.clone()).unwrap_or_default();
  let new_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  STORES.insert(new_id, StoreState { data, path: None });
  new_id
}
```
With:
```
pub fn store_clone(id: u64) -> Result<u64, String> {
  let data = STORES.get(&id).map(|s| s.data.clone()).ok_or_else(|| format!("store_clone: store {id} not found"))?;
  let new_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  STORES.insert(new_id, StoreState { data, path: None });
  Ok(new_id)
}
```

### Step 2b: Update store_clone call site in apply.rs

**File:** `/home/entropybender/repos/lx/crates/lx/src/interpreter/apply.rs`

Find and replace the following exact string:
```
        for v in fields.values_mut() {
          if let LxVal::Store { id: store_id } = v {
            *store_id = crate::stdlib::store_clone(*store_id);
          }
```
With:
```
        for v in fields.values_mut() {
          if let LxVal::Store { id: store_id } = v {
            *store_id = crate::stdlib::store_clone(*store_id).map_err(|e| LxError::runtime(e, span))?;
          }
```

## Step 3: Surface file read error in EditorView

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/terminal/view.rs`

Find and replace the following exact string:
```
    async move { if fp.is_empty() { String::new() } else { tokio::fs::read_to_string(&fp).await.unwrap_or_default() } }
```
With:
```
    async move {
      if fp.is_empty() {
        String::new()
      } else {
        match tokio::fs::read_to_string(&fp).await {
          Ok(s) => s,
          Err(e) => {
            error!("editor: failed to read {fp}: {e}");
            format!("Error reading file: {e}")
          },
        }
      }
    }
```

## Step 4: Surface file save error in EditorView

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/terminal/view.rs`

Find and replace the following exact string:
```
                let _ = tokio::fs::write(&fp, &text).await;
```
With:
```
                if let Err(e) = tokio::fs::write(&fp, &text).await {
                  error!("editor: failed to save {fp}: {e}");
                }
```

## Step 5: Surface HTTP errors on approval actions

**File:** `/home/entropybender/repos/lx/crates/lx-mobile/src/pages/approvals.rs`

This file has four `let _ = post_respond(...)` calls that silently discard HTTP failures. All four need the same treatment.

### Step 5a: Add tracing import

Find and replace the following exact string:
```
use dioxus::prelude::*;
```
With:
```
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
```

### Step 5b: Fix first post_respond call (confirm "Yes" button)

Find and replace the following exact string:
```
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(true),
                          })
                          .await;
```
With:
```
                      if let Err(e) = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(true),
                          })
                          .await
                      {
                          error!("approval respond failed: {e}");
                      }
```

### Step 5c: Fix second post_respond call (confirm "No" button)

Find and replace the following exact string:
```
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(false),
                          })
                          .await;
```
With:
```
                      if let Err(e) = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(false),
                          })
                          .await
                      {
                          error!("approval respond failed: {e}");
                      }
```

### Step 5d: Fix third post_respond call (choose option button)

Find and replace the following exact string:
```
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(i),
                          })
                          .await;
```
With:
```
                      if let Err(e) = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(i),
                          })
                          .await
                      {
                          error!("approval respond failed: {e}");
                      }
```

### Step 5e: Fix fourth post_respond call (ask "Send" button)

Find and replace the following exact string:
```
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(val),
                          })
                          .await;
```
With:
```
                      if let Err(e) = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(val),
                          })
                          .await
                      {
                          error!("approval respond failed: {e}");
                      }
```

## Step 6: Replace unwrap_or_default with expect for infallible serialization

**File:** `/home/entropybender/repos/lx/crates/lx-cli/src/agent_cmd.rs`

Find and replace the following exact string:
```
        println!("{}", serde_json::to_string(&j).unwrap_or_default());
```
With:
```
        println!("{}", serde_json::to_string(&j).expect("serde_json::Value serialization is infallible"));
```

## Step 7: Log agent spawn errors instead of swallowing them

**File:** `/home/entropybender/repos/lx/crates/lx/src/builtins/agent.rs`

Find and replace the following exact string:
```
    ctx.tokio_runtime.clone().block_on(async {
      let mut interp = crate::interpreter::Interpreter::new(&source_clone, None, ctx);
      interp.load_default_tools().await.ok();
      let _ = interp.exec(&program).await;
    });
```
With:
```
    ctx.tokio_runtime.clone().block_on(async {
      let mut interp = crate::interpreter::Interpreter::new(&source_clone, None, ctx);
      if let Err(e) = interp.load_default_tools().await {
        eprintln!("[agent:spawn] load_default_tools failed: {e}");
        return;
      }
      if let Err(e) = interp.exec(&program).await {
        eprintln!("[agent:spawn] exec failed: {e}");
      }
    });
```

# Verification

After all changes, run `just diagnose` to confirm the codebase compiles and passes clippy.
