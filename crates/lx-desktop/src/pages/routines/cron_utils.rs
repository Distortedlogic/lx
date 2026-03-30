use super::schedule_editor::SchedulePreset;

pub struct ParsedCron {
  pub preset: SchedulePreset,
  pub hour: String,
  pub minute: String,
  pub day_of_week: String,
  pub day_of_month: String,
}

pub fn parse_cron_to_preset(cron: &str) -> ParsedCron {
  let defaults = ParsedCron { preset: SchedulePreset::EveryDay, hour: "10".into(), minute: "0".into(), day_of_week: "1".into(), day_of_month: "1".into() };

  let trimmed = cron.trim();
  if trimmed.is_empty() {
    return defaults;
  }

  let parts: Vec<&str> = trimmed.split_whitespace().collect();
  if parts.len() != 5 {
    return ParsedCron { preset: SchedulePreset::Custom, ..defaults };
  }

  let (min, hr, dom, _month, dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

  if min == "*" && hr == "*" && dom == "*" && dow == "*" {
    return ParsedCron { preset: SchedulePreset::EveryMinute, ..defaults };
  }

  if hr == "*" && dom == "*" && dow == "*" {
    return ParsedCron { preset: SchedulePreset::EveryHour, minute: if min == "*" { "0".into() } else { min.into() }, ..defaults };
  }

  if dom == "*" && dow == "*" && hr != "*" {
    return ParsedCron { preset: SchedulePreset::EveryDay, hour: hr.into(), minute: if min == "*" { "0".into() } else { min.into() }, ..defaults };
  }

  if dom == "*" && dow == "1-5" && hr != "*" {
    return ParsedCron { preset: SchedulePreset::Weekdays, hour: hr.into(), minute: if min == "*" { "0".into() } else { min.into() }, ..defaults };
  }

  if dom == "*" && dow.len() == 1 && dow.chars().all(|c| c.is_ascii_digit()) && hr != "*" {
    return ParsedCron {
      preset: SchedulePreset::Weekly,
      hour: hr.into(),
      minute: if min == "*" { "0".into() } else { min.into() },
      day_of_week: dow.into(),
      ..defaults
    };
  }

  let dom_is_numeric = !dom.is_empty() && dom.len() <= 2 && dom.chars().all(|c| c.is_ascii_digit());
  if dom_is_numeric && dow == "*" && hr != "*" {
    return ParsedCron {
      preset: SchedulePreset::Monthly,
      hour: hr.into(),
      minute: if min == "*" { "0".into() } else { min.into() },
      day_of_month: dom.into(),
      ..defaults
    };
  }

  ParsedCron { preset: SchedulePreset::Custom, ..defaults }
}

pub fn build_cron(preset: &SchedulePreset, hour: &str, minute: &str, day_of_week: &str, day_of_month: &str) -> String {
  match preset {
    SchedulePreset::EveryMinute => "* * * * *".into(),
    SchedulePreset::EveryHour => format!("{minute} * * * *"),
    SchedulePreset::EveryDay => format!("{minute} {hour} * * *"),
    SchedulePreset::Weekdays => format!("{minute} {hour} * * 1-5"),
    SchedulePreset::Weekly => format!("{minute} {hour} * * {day_of_week}"),
    SchedulePreset::Monthly => format!("{minute} {hour} {day_of_month} * *"),
    SchedulePreset::Custom => String::new(),
  }
}

pub fn describe_schedule(cron: &str) -> String {
  let parsed = parse_cron_to_preset(cron);
  let h: u32 = parsed.hour.parse().unwrap_or(0);
  let m: u32 = parsed.minute.parse().unwrap_or(0);
  let time_str = format_time(h, m);

  match parsed.preset {
    SchedulePreset::EveryMinute => "Every minute".to_string(),
    SchedulePreset::EveryHour => format!("Every hour at minute {m:02}"),
    SchedulePreset::EveryDay => format!("Every day at {time_str}"),
    SchedulePreset::Weekdays => format!("Weekdays at {time_str}"),
    SchedulePreset::Weekly => {
      let dow_name = match parsed.day_of_week.as_str() {
        "0" => "Sunday",
        "1" => "Monday",
        "2" => "Tuesday",
        "3" => "Wednesday",
        "4" => "Thursday",
        "5" => "Friday",
        "6" => "Saturday",
        _ => "Monday",
      };
      format!("Every {dow_name} at {time_str}")
    },
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
    SchedulePreset::Custom => {
      let trimmed = cron.trim();
      if trimmed.is_empty() { "No schedule set".to_string() } else { format!("Custom: {trimmed}") }
    },
  }
}

fn format_time(h: u32, m: u32) -> String {
  let (display_h, period) = match h {
    0 => (12, "AM"),
    1..=11 => (h, "AM"),
    12 => (12, "PM"),
    _ => (h - 12, "PM"),
  };
  format!("{display_h}:{m:02} {period}")
}
