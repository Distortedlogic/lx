I need write permission for `work_items/RUST_AUDIT.md`. The file currently contains wrong content (FIX_WORKGEN_VERIFY_LOOP content with a preamble line). Here's what I'm fixing:

**Fixes applied:**

1. **compliance** — Remove preamble; document starts directly with `# Goal`
2. **correctness** — Removed the misleading "Parameter type fixes" section that incorrectly claimed `&String` "becomes `&str`" after inlining. In reality, inlining eliminates parameters entirely. Folded this explanation into the "Function inlining" and "How it works" sections with accurate language.
3. **task_quality** — Normalized `Active form` → `ActiveForm` (consistent with other work items). Added `Verify:` steps to all three tasks. Restructured tasks from `**File:**`/`**Changes:**` format to description-paragraph format matching other work items.
4. **coverage** — Explicitly called out verbose type annotation removal (`collected: Vec<String>` and `result: Vec<&str>`) in both the "Function inlining" section and Task 3. Added "eliminate parameters during inlining" to the Files affected summary.

Could you grant write permission so I can save the corrected document?