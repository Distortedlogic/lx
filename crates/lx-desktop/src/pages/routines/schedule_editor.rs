use dioxus::prelude::*;

use super::cron_utils::{build_cron, parse_cron_to_preset};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchedulePreset {
  EveryMinute,
  EveryHour,
  EveryDay,
  Weekdays,
  Weekly,
  Monthly,
  Custom,
}

impl SchedulePreset {
  fn from_value(v: &str) -> Self {
    match v {
      "every_minute" => Self::EveryMinute,
      "every_hour" => Self::EveryHour,
      "every_day" => Self::EveryDay,
      "weekdays" => Self::Weekdays,
      "weekly" => Self::Weekly,
      "monthly" => Self::Monthly,
      _ => Self::Custom,
    }
  }

  fn to_value(&self) -> &'static str {
    match self {
      Self::EveryMinute => "every_minute",
      Self::EveryHour => "every_hour",
      Self::EveryDay => "every_day",
      Self::Weekdays => "weekdays",
      Self::Weekly => "weekly",
      Self::Monthly => "monthly",
      Self::Custom => "custom",
    }
  }
}

const PRESETS: &[(&str, &str)] = &[
  ("every_minute", "Every minute"),
  ("every_hour", "Every hour"),
  ("every_day", "Every day"),
  ("weekdays", "Weekdays"),
  ("weekly", "Weekly"),
  ("monthly", "Monthly"),
  ("custom", "Custom (cron)"),
];

fn hour_label(h: usize) -> String {
  match h {
    0 => "12 AM".into(),
    1..=11 => format!("{h} AM"),
    12 => "12 PM".into(),
    _ => format!("{} PM", h - 12),
  }
}

const MINUTES: &[u32] = &[0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55];

const DAYS_OF_WEEK: &[(&str, &str)] = &[("1", "Mon"), ("2", "Tue"), ("3", "Wed"), ("4", "Thu"), ("5", "Fri"), ("6", "Sat"), ("0", "Sun")];

#[component]
pub fn ScheduleEditor(value: String, on_change: EventHandler<String>) -> Element {
  let parsed = parse_cron_to_preset(&value);
  let mut preset = use_signal(|| parsed.preset.clone());
  let mut hour = use_signal(|| parsed.hour.clone());
  let mut minute = use_signal(|| parsed.minute.clone());
  let mut day_of_week = use_signal(|| parsed.day_of_week.clone());
  let mut day_of_month = use_signal(|| parsed.day_of_month.clone());
  let mut custom_cron = use_signal(|| if parsed.preset == SchedulePreset::Custom { value.clone() } else { String::new() });

  let emit = move |p: &SchedulePreset, h: &str, m: &str, dow: &str, dom: &str, custom: &str| {
    if *p == SchedulePreset::Custom {
      on_change.call(custom.to_string());
    } else {
      on_change.call(build_cron(p, h, m, dow, dom));
    }
  };

  let cur_preset = preset();
  let cur_hour = hour();
  let cur_minute = minute();
  let cur_dow = day_of_week();
  let cur_dom = day_of_month();

  let select_cls = "bg-[var(--surface-container)] border border-[var(--outline-variant)] \
                      text-xs px-2 py-1.5 rounded outline-none text-[var(--on-surface)]";

  rsx! {
    div { class: "flex flex-col gap-3",
      select {
        class: "{select_cls} w-full",
        value: "{cur_preset.to_value()}",
        onchange: move |evt| {
            let new_preset = SchedulePreset::from_value(&evt.value());
            preset.set(new_preset.clone());
            if new_preset == SchedulePreset::Custom {
                custom_cron.set(value.clone());
            } else {
                emit(
                    &new_preset,
                    &hour(),
                    &minute(),
                    &day_of_week(),
                    &day_of_month(),
                    &custom_cron(),
                );
            }
        },
        for (val , label) in PRESETS {
          option { value: *val, "{label}" }
        }
      }
      if cur_preset == SchedulePreset::Custom {
        div { class: "flex flex-col gap-1.5",
          input {
            class: "{select_cls} w-full font-mono",
            placeholder: "0 10 * * *",
            value: "{custom_cron}",
            oninput: move |evt| {
                custom_cron.set(evt.value());
                on_change.call(evt.value());
            },
          }
          p { class: "text-xs text-[var(--outline)]",
            "Five fields: minute hour day-of-month month day-of-week"
          }
        }
      } else {
        div { class: "flex flex-wrap items-center gap-2",
          {
              render_pickers(
                  &PickerState {
                      preset: &cur_preset,
                      hour: &cur_hour,
                      minute: &cur_minute,
                      dow: &cur_dow,
                      dom: &cur_dom,
                  },
                  select_cls,
                  move |h| {
                      hour.set(h.clone());
                      emit(
                          &preset(),
                          &h,
                          &minute(),
                          &day_of_week(),
                          &day_of_month(),
                          &custom_cron(),
                      );
                  },
                  move |m| {
                      minute.set(m.clone());
                      emit(
                          &preset(),
                          &hour(),
                          &m,
                          &day_of_week(),
                          &day_of_month(),
                          &custom_cron(),
                      );
                  },
                  move |d| {
                      day_of_week.set(d.clone());
                      emit(&preset(), &hour(), &minute(), &d, &day_of_month(), &custom_cron());
                  },
                  move |d| {
                      day_of_month.set(d.clone());
                      emit(&preset(), &hour(), &minute(), &day_of_week(), &d, &custom_cron());
                  },
              )
          }
        }
      }
    }
  }
}

struct PickerState<'a> {
  preset: &'a SchedulePreset,
  hour: &'a str,
  minute: &'a str,
  dow: &'a str,
  dom: &'a str,
}

fn render_pickers(
  state: &PickerState<'_>,
  select_cls: &str,
  on_hour: impl FnMut(String) + Clone + 'static,
  on_minute: impl FnMut(String) + Clone + 'static,
  on_dow: impl FnMut(String) + Clone + 'static,
  mut on_dom: impl FnMut(String) + Clone + 'static,
) -> Element {
  let PickerState { preset, hour: cur_hour, minute: cur_minute, dow: cur_dow, dom: cur_dom } = *state;
  let on_minute2 = on_minute.clone();
  match preset {
    SchedulePreset::EveryMinute => rsx! {},
    SchedulePreset::EveryHour => rsx! {
      span { class: "text-xs text-[var(--outline)] uppercase", "at minute" }
      {minute_select(cur_minute, select_cls, on_minute2)}
    },
    SchedulePreset::EveryDay | SchedulePreset::Weekdays => rsx! {
      span { class: "text-xs text-[var(--outline)] uppercase", "at" }
      {hour_select(cur_hour, select_cls, on_hour)}
      span { class: "text-xs text-[var(--outline)]", ":" }
      {minute_select(cur_minute, select_cls, on_minute2)}
    },
    SchedulePreset::Weekly => rsx! {
      span { class: "text-xs text-[var(--outline)] uppercase", "at" }
      {hour_select(cur_hour, select_cls, on_hour)}
      span { class: "text-xs text-[var(--outline)]", ":" }
      {minute_select(cur_minute, select_cls, on_minute2)}
      span { class: "text-xs text-[var(--outline)] uppercase", "on" }
      div { class: "flex gap-1",
        for (val , label) in DAYS_OF_WEEK {
          {
              let active = cur_dow == *val;
              let val_owned = val.to_string();
              let mut on_dow = on_dow.clone();
              let cls = if active {
                  "h-7 px-2 text-xs rounded bg-[var(--primary)] text-[var(--on-primary)] font-semibold"
              } else {
                  "h-7 px-2 text-xs rounded border border-[var(--outline-variant)] text-[var(--on-surface)]"
              };
              rsx! {
                button { class: cls, onclick: move |_| on_dow(val_owned.clone()), "{label}" }
              }
          }
        }
      }
    },
    SchedulePreset::Monthly => rsx! {
      span { class: "text-xs text-[var(--outline)] uppercase", "at" }
      {hour_select(cur_hour, select_cls, on_hour)}
      span { class: "text-xs text-[var(--outline)]", ":" }
      {minute_select(cur_minute, select_cls, on_minute2)}
      span { class: "text-xs text-[var(--outline)] uppercase", "on day" }
      select {
        class: "{select_cls} w-[80px]",
        value: "{cur_dom}",
        onchange: move |evt| on_dom(evt.value()),
        for d in 1..=31u32 {
          option { value: "{d}", "{d}" }
        }
      }
    },
    SchedulePreset::Custom => rsx! {},
  }
}

fn hour_select(cur: &str, cls: &str, mut on_change: impl FnMut(String) + 'static) -> Element {
  rsx! {
    select {
      class: "{cls} w-[120px]",
      value: "{cur}",
      onchange: move |evt| on_change(evt.value()),
      for h in 0..24u32 {
        option { value: "{h}", "{hour_label(h as usize)}" }
      }
    }
  }
}

fn minute_select(cur: &str, cls: &str, mut on_change: impl FnMut(String) + 'static) -> Element {
  rsx! {
    select {
      class: "{cls} w-[80px]",
      value: "{cur}",
      onchange: move |evt| on_change(evt.value()),
      for m in MINUTES {
        option { value: "{m}", "{m:02}" }
      }
    }
  }
}
