# UNIT 6: Wire Dashboard Charts to Real ActivityLog Data

## Goal

Replace the fake `pseudo_random_height` chart bars in `activity_charts.rs` with real data derived from the `ActivityLog` context. Both `ActivitySummaryChart` and `EventBreakdownChart` currently render 14 bars with deterministic-but-meaningless heights. After this unit, the activity chart shows event counts per time bucket, and the breakdown chart shows event counts per `kind`.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/pages/dashboard/activity_charts.rs` | Rewrite both chart components to accept real data |
| `crates/lx-desktop/src/pages/dashboard/mod.rs` | Compute aggregations from ActivityLog and pass as props |

## Reference Files (read-only)

| File | Why |
|------|-----|
| `crates/lx-desktop/src/contexts/activity_log.rs` | ActivityLog struct, `events: Signal<VecDeque<ActivityEvent>>` |
| `crates/lx-api/src/types.rs` | `ActivityEvent { timestamp: String, kind: String, message: String }` |

---

## Current State

### `crates/lx-desktop/src/pages/dashboard/activity_charts.rs` (lines 1-48)

The file defines a `pseudo_random_height` function at line 3:
```rust
fn pseudo_random_height(i: usize) -> usize {
  (i * 17 + 5) % 80 + 10
}
```

`ActivitySummaryChart` (line 23) iterates `0..14` and renders bars using `pseudo_random_height(i)` as a percentage height. `EventBreakdownChart` (line 37) does the same thing with a different color class. Neither component accepts any props or reads any context.

### `crates/lx-desktop/src/pages/dashboard/mod.rs` (lines 62-68)

The dashboard renders both charts with no data props:
```rust
ChartCard { title: "Activity", subtitle: "Last 14 events".to_string(),
  ActivitySummaryChart {}
}
ChartCard { title: "Event Breakdown", subtitle: "By type".to_string(),
  EventBreakdownChart {}
}
```

---

## Step 1: Rewrite `activity_charts.rs`

Replace the entire file `crates/lx-desktop/src/pages/dashboard/activity_charts.rs` with:

```rust
use dioxus::prelude::*;

#[component]
pub fn ChartCard(title: String, #[props(optional)] subtitle: Option<String>, children: Element) -> Element {
  rsx! {
    div { class: "border border-gray-700 rounded-lg p-4 space-y-3",
      div {
        h3 { class: "text-xs font-medium text-gray-400", "{title}" }
        if let Some(ref sub) = subtitle {
          span { class: "text-[10px] text-gray-500", "{sub}" }
        }
      }
      {children}
    }
  }
}

#[component]
pub fn ActivitySummaryChart(buckets: Vec<usize>) -> Element {
  let max_val = buckets.iter().copied().max().unwrap_or(1).max(1);
  rsx! {
    div { class: "flex items-end gap-[3px] h-20",
      for (i , count) in buckets.iter().enumerate() {
        {
            let pct = (*count as f64 / max_val as f64 * 100.0) as usize;
            let pct = pct.max(2);
            rsx! {
              div {
                key: "{i}",
                class: "flex-1 bg-gray-700/30 rounded-sm",
                style: "height: {pct}%",
              }
            }
        }
      }
    }
  }
}

#[component]
pub fn EventBreakdownChart(segments: Vec<(String, usize)>) -> Element {
  let max_val = segments.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);
  let colors = ["bg-emerald-700/30", "bg-cyan-700/30", "bg-amber-700/30", "bg-red-700/30", "bg-violet-700/30"];
  rsx! {
    div { class: "flex items-end gap-[3px] h-20",
      for (i , (_label , count)) in segments.iter().enumerate() {
        {
            let pct = (*count as f64 / max_val as f64 * 100.0) as usize;
            let pct = pct.max(2);
            let color = colors[i % colors.len()];
            rsx! {
              div {
                key: "{i}",
                class: "flex-1 {color} rounded-sm relative group",
                style: "height: {pct}%",
                div { class: "absolute -top-5 left-1/2 -translate-x-1/2 text-[9px] text-gray-400 opacity-0 group-hover:opacity-100 whitespace-nowrap",
                  "{_label}: {count}"
                }
              }
            }
        }
      }
    }
    if !segments.is_empty() {
      div { class: "flex flex-wrap gap-x-3 gap-y-1 mt-2",
        for (i , (label , count)) in segments.iter().enumerate() {
          div { class: "flex items-center gap-1",
            span {
              class: "w-2 h-2 rounded-sm {colors[i % colors.len()]}",
            }
            span { class: "text-[10px] text-gray-400", "{label} ({count})" }
          }
        }
      }
    }
  }
}
```

**What changed:**
- Deleted `pseudo_random_height` function entirely.
- `ActivitySummaryChart` now takes `buckets: Vec<usize>` prop. Each entry is an event count for one time bucket. Bar height = `count / max * 100%`, clamped to minimum 2%.
- `EventBreakdownChart` now takes `segments: Vec<(String, usize)>` prop. Each entry is `(kind_label, count)`. Uses rotating color palette. Shows hover tooltip and legend row.
- `ChartCard` is unchanged.

---

## Step 2: Compute aggregations in `mod.rs` and pass as props

In `crates/lx-desktop/src/pages/dashboard/mod.rs`, make two changes.

### Change 2a: Add import for HashMap

Old text (line 1-12):
```rust
pub mod active_agents_panel;
pub mod activity_charts;

use dioxus::prelude::*;

use crate::components::empty_state::EmptyState;
use crate::components::metric_card::MetricCard;
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::breadcrumb::BreadcrumbEntry;

use self::active_agents_panel::ActiveAgentsPanel;
use self::activity_charts::{ActivitySummaryChart, ChartCard, EventBreakdownChart};
```

New text:
```rust
pub mod active_agents_panel;
pub mod activity_charts;

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::components::empty_state::EmptyState;
use crate::components::metric_card::MetricCard;
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::breadcrumb::BreadcrumbEntry;

use self::active_agents_panel::ActiveAgentsPanel;
use self::activity_charts::{ActivitySummaryChart, ChartCard, EventBreakdownChart};
```

### Change 2b: Build aggregations and pass props to charts

Old text (lines 60-68):
```rust
      div { class: "grid grid-cols-2 lg:grid-cols-4 gap-4",
        ChartCard { title: "Activity", subtitle: "Last 14 events".to_string(),
          ActivitySummaryChart {}
        }
        ChartCard { title: "Event Breakdown", subtitle: "By type".to_string(),
          EventBreakdownChart {}
        }
      }
```

New text:
```rust
      div { class: "grid grid-cols-2 lg:grid-cols-4 gap-4",
        ChartCard { title: "Activity", subtitle: format!("Last {} buckets", activity_buckets.len()),
          ActivitySummaryChart { buckets: activity_buckets.clone() }
        }
        ChartCard { title: "Event Breakdown", subtitle: "By type".to_string(),
          EventBreakdownChart { segments: breakdown_segments.clone() }
        }
      }
```

### Change 2c: Compute `activity_buckets` and `breakdown_segments` after the metric counts

Old text (lines 27-28):
```rust
  let error_events = events.iter().filter(|e| e.kind.to_lowercase().contains("error") || e.message.to_lowercase().contains("error")).count();

  if total_events == 0 {
```

New text:
```rust
  let error_events = events.iter().filter(|e| e.kind.to_lowercase().contains("error") || e.message.to_lowercase().contains("error")).count();

  let activity_buckets: Vec<usize> = {
    let bucket_count = 14usize;
    let total = events.len();
    if total == 0 {
      vec![0; bucket_count]
    } else {
      let per_bucket = (total + bucket_count - 1) / bucket_count;
      let mut buckets = Vec::with_capacity(bucket_count);
      for chunk in events.iter().collect::<Vec<_>>().chunks(per_bucket.max(1)) {
        buckets.push(chunk.len());
      }
      while buckets.len() < bucket_count {
        buckets.push(0);
      }
      buckets.truncate(bucket_count);
      buckets.reverse();
      buckets
    }
  };

  let breakdown_segments: Vec<(String, usize)> = {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for event in events.iter() {
      let key = event.kind.split('_').next().unwrap_or(&event.kind).to_string();
      *counts.entry(key).or_default() += 1;
    }
    let mut segments: Vec<(String, usize)> = counts.into_iter().collect();
    segments.sort_by(|a, b| b.1.cmp(&a.1));
    segments.truncate(8);
    segments
  };

  if total_events == 0 {
```

**What this does:**
- `activity_buckets`: Divides the events VecDeque into 14 equal-sized time buckets (newest events are at front of VecDeque, so we reverse to show oldest-to-newest left-to-right). Each bucket value is the count of events in that slice.
- `breakdown_segments`: Groups events by the first segment of their `kind` field (e.g., `agent_start` -> `agent`, `tool_call` -> `tool`). Sorts descending by count. Truncates to 8 categories max.

---

## Verification

After all changes:
- `activity_charts.rs` is ~75 lines (under 300).
- `mod.rs` is ~125 lines (under 300).
- No code comments or docstrings.
- No `#[allow(...)]` macros.
- Dashboard with 0 events still shows the EmptyState (the aggregation code runs but the early-return on line `if total_events == 0` still fires before rendering).
- Dashboard with events renders real bar heights proportional to actual event counts.
- Hover on breakdown bars shows `kind: count` tooltip.
