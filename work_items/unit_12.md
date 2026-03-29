# Unit 12: Costs & Approvals Pages

Port the Costs/Budgets page and Approvals list/detail pages from Paperclip React to Dioxus 0.7.3 in lx-desktop.

## Paperclip Source Files

| Paperclip File | Purpose |
|---|---|
| `reference/paperclip/ui/src/pages/Costs.tsx` | Costs overview with tabs: overview, budgets, providers |
| `reference/paperclip/ui/src/components/BudgetPolicyCard.tsx` | Budget policy display with progress bar and save controls |
| `reference/paperclip/ui/src/components/BudgetIncidentCard.tsx` | Budget incident card with raise-and-resume flow |
| `reference/paperclip/ui/src/components/ProviderQuotaCard.tsx` | Provider spend card with token/cost breakdowns |
| `reference/paperclip/ui/src/components/BillerSpendCard.tsx` | Biller spend card with upstream provider breakdown |
| `reference/paperclip/ui/src/components/AccountingModelCard.tsx` | Static info card explaining the accounting model |
| `reference/paperclip/ui/src/pages/Approvals.tsx` | Approvals list with pending/all filter tabs |
| `reference/paperclip/ui/src/pages/ApprovalDetail.tsx` | Approval detail with actions, comments, linked issues |
| `reference/paperclip/ui/src/components/ApprovalCard.tsx` | Individual approval card with approve/reject buttons |
| `reference/paperclip/ui/src/components/ApprovalPayload.tsx` | Type-specific payload renderers for approval types |

## Preconditions

1. **Unit 3 is complete:** Unit 3 created stubs `pages/costs.rs` and `pages/approvals.rs`. This unit replaces them with real modules. Delete each stub file and create directory modules at those paths (e.g., `src/pages/costs/mod.rs` and `src/pages/approvals/mod.rs`). The `routes.rs` Route enum already has `Costs {}`, `Approvals {}`, and `ApprovalDetail { approval_id: String }` variants importing from `crate::pages::costs` and `crate::pages::approvals` -- no changes to `routes.rs` are needed.
2. Units 10 and 11 are complete: pages/mod.rs declares all modules; sidebar is fully extended
3. `crates/lx-desktop/src/styles.rs` has `PAGE_HEADING` and `FLEX_BETWEEN`
4. Precondition verified: `dioxus-storage` is already a dependency in Cargo.toml.
5. `uuid` crate with `v4` feature is available (added in Unit 10)

## Data Types

Create `crates/lx-desktop/src/pages/costs/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetPolicy {
    pub id: String,
    pub scope_type: String,
    pub scope_id: String,
    pub scope_name: String,
    pub amount_cents: u64,
    pub observed_cents: u64,
    pub warn_percent: u32,
    pub hard_stop: bool,
    pub status: String,
    pub paused: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderSpend {
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_cents: u64,
}
```

Create `crates/lx-desktop/src/pages/approvals/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub approval_type: String,
    pub status: String,
    pub requested_by: Option<String>,
    pub payload: ApprovalPayload,
    pub decision_note: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalPayload {
    pub name: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub amount: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalComment {
    pub id: String,
    pub body: String,
    pub author: Option<String>,
    pub created_at: String,
}
```

## File Plan

| New File | Lines (est.) | Purpose |
|---|---|---|
| `crates/lx-desktop/src/pages/costs/types.rs` | ~30 | BudgetPolicy and ProviderSpend structs |
| `crates/lx-desktop/src/pages/costs/mod.rs` | ~15 | Module declarations, re-exports |
| `crates/lx-desktop/src/pages/costs/overview.rs` | ~120 | Costs overview page with tabs |
| `crates/lx-desktop/src/pages/costs/budget_card.rs` | ~130 | Budget policy card with progress bar |
| `crates/lx-desktop/src/pages/costs/provider_card.rs` | ~100 | Provider spend card with model breakdown |
| `crates/lx-desktop/src/pages/costs/accounting_card.rs` | ~60 | Static accounting model info card |
| `crates/lx-desktop/src/pages/approvals/types.rs` | ~35 | Approval, ApprovalPayload, ApprovalComment structs |
| `crates/lx-desktop/src/pages/approvals/mod.rs` | ~15 | Module declarations, re-exports |
| `crates/lx-desktop/src/pages/approvals/list.rs` | ~130 | Approvals list page with pending/all tabs |
| `crates/lx-desktop/src/pages/approvals/detail.rs` | ~150 | Approval detail page shell |
| `crates/lx-desktop/src/pages/approvals/approval_actions.rs` | ~150 | Action buttons, revision form, comments section |
| `crates/lx-desktop/src/pages/approvals/card.rs` | ~90 | Individual approval card component |
| `crates/lx-desktop/src/pages/approvals/payload.rs` | ~80 | Type-specific payload renderers |

## Step 1: Create `crates/lx-desktop/src/pages/costs/types.rs`

Create the file with the `BudgetPolicy` and `ProviderSpend` structs as specified above.

Add these helper functions:

```rust
pub fn format_cents(cents: u64) -> String {
    let dollars = cents / 100;
    let remainder = cents % 100;
    format!("${}.{:02}", dollars, remainder)
}

pub fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        format!("{}", tokens)
    }
}

pub fn utilization_percent(observed: u64, budget: u64) -> u32 {
    if budget == 0 { return 0; }
    ((observed as f64 / budget as f64) * 100.0).min(100.0) as u32
}
```

## Step 2: Create `crates/lx-desktop/src/pages/costs/mod.rs`

```rust
mod accounting_card;
mod budget_card;
mod overview;
mod provider_card;
pub mod types;

pub use overview::Costs;
```

## Step 3: Create `crates/lx-desktop/src/pages/costs/budget_card.rs`

This component mirrors `BudgetPolicyCard.tsx`. It displays a budget policy with a progress bar and optional save controls.

Structure:
- Props: `policy: BudgetPolicy`, `on_save: Option<EventHandler<u64>>` (optional callback with new amount in cents)
- Local signal: `draft_budget: Signal<String>` initialized from `format!("{:.2}", policy.amount_cents as f64 / 100.0)`
- Render:
  - Header section:
    - Scope type as uppercase muted label ("PROJECT", "AGENT", etc.)
    - Scope name as bold title
    - Status indicator on the right:
      - If `policy.status == "hard_stop"`: red badge text "HARD STOP" with `shield_alert` icon (material symbol "gpp_maybe")
      - If `policy.status == "warning"`: amber badge text "WARNING"
      - Otherwise: green text "HEALTHY"
  - Observed/Budget grid (two columns):
    - Column 1: "OBSERVED" label (11px uppercase tracking), observed amount as xl bold text (`format_cents`), utilization percent below
    - Column 2: "BUDGET" label, budget amount as xl bold text (or "DISABLED" if 0), warn percent below
  - Progress bar:
    - "Remaining" label with remaining amount on the right
    - A bar div (h-2 rounded-full), background `bg-[var(--outline-variant)]`, inner fill div with width `{utilization_percent}%`
    - Fill color: red if hard_stop, amber if warning, green otherwise
  - If paused: a red-bordered info box with pause icon text: "Execution is paused until the budget is raised"
  - If `on_save` is Some:
    - Dollar input field: text input with `inputmode="decimal"`, placeholder "0.00", bound to `draft_budget`
    - Save button: parse `draft_budget` to cents, call `on_save` if valid and different from current amount
    - Button label: "SET BUDGET" if amount_cents is 0, "UPDATE BUDGET" otherwise
    - Disable button if parsed amount equals current or parse fails
    - If parse fails, show red error text: "Enter a valid non-negative dollar amount"

Style with lx-desktop design:
- Card: `bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-5`
- Labels: `text-[10px] uppercase tracking-[0.18em] text-[var(--outline)]`
- Values: `text-xl font-semibold text-[var(--on-surface)]`

## Step 4: Create `crates/lx-desktop/src/pages/costs/provider_card.rs`

This component mirrors `ProviderQuotaCard.tsx` (simplified).

Structure:
- Props: `provider: String`, `rows: Vec<ProviderSpend>`
- Compute totals: sum `input_tokens`, `output_tokens`, `cost_cents` across all rows
- Render:
  - Header: provider name as bold title, total cost as xl bold number on the right
  - Subtitle: total input tokens + "in" / total output tokens + "out" (using `format_tokens`)
  - Model breakdown section (border-t separator):
    - For each row:
      - Model name (monospace, muted)
      - Token count + cost on the right
      - A token-share bar: `h-2` bar, fill width as percent of total tokens

Style with lx-desktop design:
- Card: `bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-4`

## Step 5: Create `crates/lx-desktop/src/pages/costs/accounting_card.rs`

This component mirrors `AccountingModelCard.tsx`. It is a static info card.

Structure:
- No props
- Render a card with three surface sections in a 3-column grid (stack on small screens):
  - "Inference ledger": icon `database` (material symbol), description "Request-scoped usage and billed runs", bullet points: "tokens + billed dollars", "provider, biller, model", "subscription and overage aware"
  - "Finance ledger": icon `receipt_long`, description "Account-level charges not tied to a single request", bullet points: "top-ups, refunds, fees", "provisioned charges", "credit expiries"
  - "Live quotas": icon `speed`, description "Provider windows that can stop traffic in real time", bullet points: "provider quota windows", "biller credit systems", "errors surfaced directly"
- Each section styled as a bordered rounded div with the icon in a circle, title + description, and bullet points

## Step 6: Create `crates/lx-desktop/src/pages/costs/overview.rs`

This component mirrors `Costs.tsx` (simplified).

Structure:
- Use `dioxus_storage::use_persistent("lx_budget_policies", || default_budget_policies())` for budget data
- Use `dioxus_storage::use_persistent("lx_provider_spend", || default_provider_spend())` for provider data
- `default_budget_policies()` returns a `Vec<BudgetPolicy>` with 2-3 sample policies (e.g., "Company" scope with $100 budget, "Project Alpha" with $50)
- `default_provider_spend()` returns a `Vec<ProviderSpend>` with 3-4 sample rows (e.g., anthropic/claude-sonnet, openai/gpt-4o)
- Local signal: `active_tab: Signal<&'static str>` defaulting to `"overview"`
- Render:
  - Page header: "COSTS" heading
  - Tab bar: "OVERVIEW", "BUDGETS", "PROVIDERS" buttons
  - If `active_tab == "overview"`:
    - Summary metrics in a 3-column grid:
      - "TOTAL SPEND": sum of all provider spend cost_cents, using `format_cents`
      - "PROVIDERS": count of unique providers
      - "MODELS": count of total rows
    - `AccountingModelCard {}` below the metrics
  - If `active_tab == "budgets"`:
    - For each budget policy, render `BudgetCard { policy, on_save: handler }`
    - The `on_save` handler updates the `amount_cents` of the corresponding policy in storage and recomputes `status` based on the new threshold (if observed > amount: "hard_stop"; if observed > amount * warn_percent / 100: "warning"; else "ok")
  - If `active_tab == "providers"`:
    - Group `provider_spend` rows by provider name
    - For each provider group, render `ProviderCard { provider, rows }`

## Step 7: Create `crates/lx-desktop/src/pages/approvals/types.rs`

Create the file with the `Approval`, `ApprovalPayload`, and `ApprovalComment` structs as specified in the Data Types section above.

Add these constants:

```rust
pub const APPROVAL_TYPES: &[(&str, &str)] = &[
    ("hire_agent", "Hire Agent"),
    ("approve_ceo_strategy", "CEO Strategy"),
    ("budget_override_required", "Budget Override"),
];

pub fn approval_type_label(t: &str) -> &str {
    APPROVAL_TYPES
        .iter()
        .find(|(k, _)| *k == t)
        .map(|(_, v)| *v)
        .unwrap_or(t)
}

pub fn approval_type_icon(t: &str) -> &'static str {
    match t {
        "hire_agent" => "person_add",
        "approve_ceo_strategy" => "lightbulb",
        "budget_override_required" => "gpp_maybe",
        _ => "verified_user",
    }
}
```

## Step 8: Create `crates/lx-desktop/src/pages/approvals/mod.rs`

```rust
mod approval_actions;
mod card;
mod detail;
mod list;
mod payload;
pub mod types;

pub use detail::ApprovalDetail;
pub use list::Approvals;
```

## Step 9: Create `crates/lx-desktop/src/pages/approvals/payload.rs`

This component mirrors `ApprovalPayload.tsx`. It renders type-specific payload information.

Structure:
- `PayloadRenderer` component: props `approval_type: String`, `payload: ApprovalPayload`
- Render differs by type:
  - If `approval_type == "hire_agent"`:
    - Property rows: "Name" -> payload.name, "Role" -> payload.role, "Title" -> payload.title
    - Each row: label (muted, w-20, xs) + value text
  - If `approval_type == "budget_override_required"`:
    - "Scope" -> payload.name, "Amount" -> payload.amount formatted with `format_cents`
    - If description present: a muted code block with the description
  - Otherwise (CEO strategy / generic):
    - "Title" -> payload.title
    - If description: a muted pre block with the description text
- `PayloadField` helper component: props `label: &str`, `value: Option<String>`
  - If value is None, return `None`
  - Render a flex row: label span (w-20, muted) + value span

## Step 10: Create `crates/lx-desktop/src/pages/approvals/card.rs`

This component mirrors `ApprovalCard.tsx`. It renders a single approval as a card.

Structure:
- Props: `approval: Approval`, `on_approve: EventHandler<()>`, `on_reject: EventHandler<()>`, `is_pending: bool`
- Render:
  - Card container: `border border-[var(--outline-variant)] p-4`
  - Header row:
    - Left: material icon from `approval_type_icon`, label from `approval_type_label`, plus contextual name (e.g., "Hire Agent: Designer" if `approval.payload.name` is Some)
    - Right: status icon + status text + relative time
      - Status icons: "check_circle" green for approved, "cancel" red for rejected, "schedule" yellow for pending, "schedule" amber for revision_requested
  - Below header: `PayloadRenderer { approval_type, payload }`
  - If `approval.decision_note` is Some: italic muted text with the note
  - Action buttons (shown only if status is "pending" or "revision_requested" and type is not "budget_override_required"):
    - "APPROVE" button: green bg, calls `on_approve`
    - "REJECT" button: red/destructive, calls `on_reject`
    - Both disabled when `is_pending` is true
  - "VIEW DETAILS" link to `Route::ApprovalDetail { id }`

## Step 11: Create `crates/lx-desktop/src/pages/approvals/list.rs`

This component mirrors `Approvals.tsx`.

Structure:
- Use `dioxus_storage::use_persistent("lx_approvals", || default_approvals())` for storage
- `default_approvals()` returns a `Vec<Approval>` with 2-3 sample approvals:
  - A pending "hire_agent" approval with payload name "Designer"
  - An approved "approve_ceo_strategy" approval
  - A pending "budget_override_required" approval
- Local signals: `status_filter: Signal<&'static str>` (default "pending"), `action_error: Signal<Option<String>>`
- Compute `pending_count`: count approvals where status is "pending" or "revision_requested"
- Compute `filtered`: if filter is "pending", keep pending/revision_requested; if "all", keep all; sort by created_at descending
- Render:
  - Tab bar at top: "PENDING" button (with count badge if > 0) and "ALL" button
    - Pending count badge: a small rounded span with the count, yellow background
  - If `action_error` is Some: red error text
  - If `filtered` is empty: centered empty state with `verified_user` icon and "No pending approvals" or "No approvals yet"
  - If `filtered` is non-empty: grid of `ApprovalCard` components, one per approval
    - `on_approve` handler: find approval in storage by id, set status to "approved", clear action_error
    - `on_reject` handler: find approval in storage by id, set status to "rejected", clear action_error
    - `is_pending`: false (no network call in mock)

## Step 12: Create `crates/lx-desktop/src/pages/approvals/detail.rs`

This component mirrors `ApprovalDetail.tsx`.

Structure:
- Read `id: String` from route params
- Use `dioxus_storage::use_persistent("lx_approvals", ...)` for approvals
- Use `dioxus_storage::use_persistent("lx_approval_comments", || HashMap::<String, Vec<ApprovalComment>>::new())` for comments. Store all comments in a single `HashMap<String, Vec<ApprovalComment>>` keyed by approval ID, and filter by the current approval's ID at render time.
- Find approval by `id`; if not found, render "Approval not found"
- Local signals: `comment_body: Signal<String>`, `error: Signal<Option<String>>`, `show_raw: Signal<bool>`
- Render:
  - Main card (bordered, rounded, p-4):
    - Header row: type icon + label (from `approval_type_icon` / `approval_type_label`), approval id (first 8 chars, monospace), status badge on right
    - Requester line: "Requested by" + `approval.requested_by` name (or "Unknown")
    - `PayloadRenderer { approval_type, payload }`
    - "See full request" toggle: a button with chevron icon, clicking toggles `show_raw`
      - When show_raw is true: render a `<pre>` block with the payload debug-printed
    - If decision_note is Some: muted italic text
    - If error is Some: red error text
    - Action buttons (only if status is "pending" or "revision_requested"):
      - If type is NOT "budget_override_required":
        - "APPROVE" button: green, sets status to "approved" in storage
        - "REJECT" button: red, sets status to "rejected" in storage
      - If type is "budget_override_required" and status is "pending":
        - Text: "Resolve this budget stop from the budget controls on /costs"
      - If status is "pending": "REQUEST REVISION" button, sets status to "revision_requested"
      - If status is "revision_requested": "MARK RESUBMITTED" button, sets status to "pending"
  - Comments card (bordered, rounded, p-4, below main card):
    - Header: "COMMENTS ({count})"
    - For each comment:
      - Bordered div with author name (or "Board"), timestamp, and body text
    - Textarea bound to `comment_body`, placeholder "Add a comment..."
    - "POST COMMENT" button: generates UUID for comment id, pushes new `ApprovalComment` with current timestamp, clears `comment_body`

## Step 13: Verify `crates/lx-desktop/src/pages/mod.rs`

The `pub mod costs;` and `pub mod approvals;` declarations already exist from Unit 3. No changes needed.

## Step 14: Note on routes

Unit 3 already has `Costs`, `Approvals`, and `ApprovalDetail` route variants with imports pointing at `crate::pages::costs` and `crate::pages::approvals`. Creating the real directory modules at those paths replaces the stubs automatically. Do NOT modify `routes.rs` or `pages/mod.rs`.

## Step 15: Update `crates/lx-desktop/src/layout/sidebar.rs`

Add two new `NavItem` entries after ORG and before SETTINGS:

```rust
NavItem {
    to: Route::Costs {},
    label: "COSTS",
    icon: "payments",
}
NavItem {
    to: Route::Approvals {},
    label: "APPROVALS",
    icon: "verified_user",
}
```

## Definition of Done

1. `just diagnose` passes with no errors and no warnings
2. Sidebar shows COSTS and APPROVALS nav items
3. Clicking COSTS shows the costs overview page with OVERVIEW, BUDGETS, and PROVIDERS tabs
4. OVERVIEW tab shows total spend, provider count, model count, and the accounting model info card
5. BUDGETS tab shows budget policy cards with progress bars, observed/budget amounts, and status indicators
6. Budget cards have a working dollar input and "UPDATE BUDGET" button that recalculates status
7. PROVIDERS tab shows provider cards grouped by provider with model breakdowns and token-share bars
8. Clicking APPROVALS shows the approvals list with PENDING and ALL filter tabs
9. The pending tab shows a count badge when pending approvals exist
10. Each approval card shows type icon, label, status, and action buttons
11. APPROVE and REJECT buttons work and update the approval status in storage
12. Clicking "VIEW DETAILS" navigates to the approval detail page
13. The approval detail page shows full payload, raw request toggle, action buttons, and comments
14. Comments can be posted and appear in the list with author and timestamp
15. Sample/default data populates costs and approvals on first load
16. No file exceeds 300 lines
