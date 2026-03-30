# WU-10: Schedule editor ordinal suffixes

## Fixes
- Fix 1: Add an `ordinal_suffix(n: u32) -> &'static str` function to cron_utils.rs
- Fix 2: Add an `ordinal(n: u32) -> String` function to cron_utils.rs that combines number + suffix
- Fix 3: Use ordinal labels in the monthly day-of-month Select options in schedule_editor.rs (e.g. "1st", "2nd", "3rd", "4th"..."31st")
- Fix 4: Use ordinal in `describe_schedule` for the Monthly preset description
- Fix 5-15: Ensure all 31 day values (1..=31) produce correct ordinal suffixes: 1st, 2nd, 3rd, 4th-20th, 21st, 22nd, 23rd, 24th-30th, 31st

## Files Modified
- `crates/lx-desktop/src/pages/routines/cron_utils.rs` (128 lines)
- `crates/lx-desktop/src/pages/routines/schedule_editor.rs` (279 lines)

## Preconditions
- `cron_utils.rs` already has an ordinal suffix block in `describe_schedule` at lines 104-109 handling 1/21/31=>"st", 2/22=>"nd", 3/23=>"rd", _=>"th"
- The monthly date picker in `schedule_editor.rs` is at lines 244-249 inside `render_pickers`, in the `SchedulePreset::Monthly` arm
- The Select options are generated with: `(1..=31u32).map(|d| SelectOption::new(d.to_string(), d.to_string())).collect::<Vec<_>>()`
- The `Select` component displays `SelectOption.label` as the visible text and uses `SelectOption.value` as the underlying value
- `describe_schedule` at line 103 already computes ordinal suffix locally but does not use a shared function

## Steps

### Step 1: Add ordinal_suffix function to cron_utils.rs
- Open `crates/lx-desktop/src/pages/routines/cron_utils.rs`
- After the `format_time` function (after line 128, the closing `}` of `format_time`), add:

```rust
pub fn ordinal_suffix(n: u32) -> &'static str {
  match (n % 10, n % 100) {
    (1, 11) => "th",
    (2, 12) => "th",
    (3, 13) => "th",
    (1, _) => "st",
    (2, _) => "nd",
    (3, _) => "rd",
    _ => "th",
  }
}

pub fn ordinal(n: u32) -> String {
  format!("{n}{}", ordinal_suffix(n))
}
```

- Why: A shared ordinal function eliminates the inline match at line 104 and makes it available to the schedule editor for day labels

### Step 2: Use ordinal() in describe_schedule
- Open `crates/lx-desktop/src/pages/routines/cron_utils.rs`
- Find lines 103-109:

```rust
    SchedulePreset::Monthly => {
      let dom: u32 = parsed.day_of_month.parse().unwrap_or(1);
      let suffix = match dom {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th",
      };
      format!("Monthly on the {dom}{suffix} at {time_str}")
    },
```

Replace with:

```rust
    SchedulePreset::Monthly => {
      let dom: u32 = parsed.day_of_month.parse().unwrap_or(1);
      format!("Monthly on the {} at {time_str}", ordinal(dom))
    },
```

- Why: Uses the shared function instead of duplicating the suffix logic

### Step 3: Use ordinal labels in monthly day-of-month Select
- Open `crates/lx-desktop/src/pages/routines/schedule_editor.rs`
- At line 1, add import: change `use super::cron_utils::{build_cron, describe_schedule, parse_cron_to_preset};` to `use super::cron_utils::{build_cron, describe_schedule, ordinal, parse_cron_to_preset};`
- Find lines 244-249 in the `SchedulePreset::Monthly` arm:

```rust
      Select {
        class: "{select_cls} w-[80px]",
        value: cur_dom.to_string(),
        options: (1..=31u32).map(|d| SelectOption::new(d.to_string(), d.to_string())).collect::<Vec<_>>(),
        onchange: move |val: String| on_dom(val),
      }
```

Replace with:

```rust
      Select {
        class: "{select_cls} w-[80px]",
        value: cur_dom.to_string(),
        options: (1..=31u32).map(|d| SelectOption::new(d.to_string(), ordinal(d))).collect::<Vec<_>>(),
        onchange: move |val: String| on_dom(val),
      }
```

- Why: The dropdown label changes from bare numbers ("1", "2", "3") to ordinal strings ("1st", "2nd", "3rd"), which is the expected UX for day-of-month selection

## File Size Check
- `cron_utils.rs`: was 128 lines, now ~140 lines (under 300)
- `schedule_editor.rs`: was 279 lines, now ~280 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compile errors or warnings
- The monthly day-of-month dropdown should display "1st", "2nd", "3rd", "4th", ... "11th", "12th", "13th", ... "21st", "22nd", "23rd", ... "31st"
- The schedule description for monthly should read e.g. "Monthly on the 1st at 10:00 AM" instead of "Monthly on the 1st at 10:00 AM" (verify it still works, no regression)
- Verify edge cases: 11th, 12th, 13th must use "th" (not "st", "nd", "rd")
- The `value` sent via `onchange` must still be the raw number string (e.g. "1", "2") — only the display `label` changes
