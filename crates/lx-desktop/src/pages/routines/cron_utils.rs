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
