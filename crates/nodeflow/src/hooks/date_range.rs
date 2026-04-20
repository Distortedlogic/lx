use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePreset {
  Mtd,
  Last7d,
  Last30d,
  Ytd,
  All,
  Custom,
}

impl DatePreset {
  pub fn label(&self) -> &'static str {
    match self {
      Self::Mtd => "Month to Date",
      Self::Last7d => "Last 7 Days",
      Self::Last30d => "Last 30 Days",
      Self::Ytd => "Year to Date",
      Self::All => "All Time",
      Self::Custom => "Custom",
    }
  }

  pub fn all() -> &'static [DatePreset] {
    &[Self::Mtd, Self::Last7d, Self::Last30d, Self::Ytd, Self::All, Self::Custom]
  }
}

pub struct DateRangeState {
  pub preset: Signal<DatePreset>,
  pub custom_from: Signal<String>,
  pub custom_to: Signal<String>,
  pub from: Memo<String>,
  pub to: Memo<String>,
  pub custom_ready: Memo<bool>,
}

struct UtcDate {
  year: i64,
  month: u32,
  day: u32,
  hour: u32,
  min: u32,
  sec: u32,
}

impl UtcDate {
  fn from_timestamp(ts: i64) -> Self {
    let days = ts.div_euclid(86400);
    let time_of_day = ts.rem_euclid(86400) as u32;
    let hour = time_of_day / 3600;
    let min = (time_of_day % 3600) / 60;
    let sec = time_of_day % 60;

    let (year, month, day) = civil_from_days(days);
    Self { year, month, day, hour, min, sec }
  }

  fn to_iso(&self) -> String {
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", self.year, self.month, self.day, self.hour, self.min, self.sec)
  }
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
  let z = z + 719468;
  let era = z.div_euclid(146097);
  let doe = z.rem_euclid(146097) as u32;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
  let y = yoe as i64 + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let d = doy - (153 * mp + 2) / 5 + 1;
  let m = if mp < 10 { mp + 3 } else { mp - 9 };
  let y = if m <= 2 { y + 1 } else { y };
  (y, m, d)
}

fn now_timestamp() -> i64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs() as i64
}

pub fn compute_range(preset: DatePreset) -> (String, String) {
  let ts = now_timestamp();
  let now = UtcDate::from_timestamp(ts);
  let to = now.to_iso();

  match preset {
    DatePreset::Mtd => {
      let from = UtcDate { year: now.year, month: now.month, day: 1, hour: 0, min: 0, sec: 0 };
      (from.to_iso(), to)
    },
    DatePreset::Last7d => {
      let from_ts = ts - 7 * 86400;
      (UtcDate::from_timestamp(from_ts).to_iso(), to)
    },
    DatePreset::Last30d => {
      let from_ts = ts - 30 * 86400;
      (UtcDate::from_timestamp(from_ts).to_iso(), to)
    },
    DatePreset::Ytd => {
      let from = UtcDate { year: now.year, month: 1, day: 1, hour: 0, min: 0, sec: 0 };
      (from.to_iso(), to)
    },
    DatePreset::All => (String::from("1970-01-01T00:00:00Z"), to),
    DatePreset::Custom => (String::new(), String::new()),
  }
}

pub fn use_date_range() -> DateRangeState {
  let preset = use_signal(|| DatePreset::Last30d);
  let custom_from = use_signal(String::new);
  let custom_to = use_signal(String::new);
  let mut tick = use_signal(|| 0u64);

  use_future(move || async move {
    loop {
      let now_secs = now_timestamp();
      let secs_into_minute = now_secs % 60;
      let sleep_secs = 60 - secs_into_minute;
      tokio::time::sleep(Duration::from_secs(sleep_secs as u64)).await;
      tick.set((tick)() + 1);
    }
  });

  let from = use_memo(move || {
    let _ = (tick)();
    let p = (preset)();
    if p == DatePreset::Custom {
      return (custom_from)();
    }
    let (f, _) = compute_range(p);
    f
  });

  let to = use_memo(move || {
    let _ = (tick)();
    let p = (preset)();
    if p == DatePreset::Custom {
      return (custom_to)();
    }
    let (_, t) = compute_range(p);
    t
  });

  let custom_ready = use_memo(move || {
    let p = (preset)();
    if p != DatePreset::Custom {
      return true;
    }
    !(custom_from)().is_empty() && !(custom_to)().is_empty()
  });

  DateRangeState { preset, custom_from, custom_to, from, to, custom_ready }
}
