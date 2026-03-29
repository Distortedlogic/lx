use dioxus::prelude::*;

fn hash_string(value: &str) -> u32 {
  let mut hash: u32 = 2166136261;
  for byte in value.bytes() {
    hash ^= byte as u32;
    hash = hash.wrapping_mul(16777619);
  }
  hash
}

fn hash_to_hsl(hash: u32) -> (u16, u8, u8) {
  let hue = (hash % 360) as u16;
  let sat = 50 + (hash / 360 % 20) as u8;
  let light = 35 + (hash / 7200 % 15) as u8;
  (hue, sat, light)
}

#[component]
pub fn CompanyPatternIcon(company_name: String, brand_color: Option<String>, class: Option<String>) -> Element {
  let trimmed = company_name.trim();
  let initial = trimmed.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "?".to_string());

  let hash = hash_string(&trimmed.to_lowercase());
  let (hue, sat, light) = if let Some(ref color) = brand_color {
    if color.starts_with('#') && color.len() == 7 {
      let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(100);
      let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(100);
      let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(200);
      let max = r.max(g).max(b);
      let min = r.min(g).min(b);
      let d = max - min;
      let h = if d == 0 {
        0u16
      } else if max == r {
        (60.0 * (((g as f32 - b as f32) / d as f32) % 6.0)) as u16
      } else if max == g {
        (60.0 * (((b as f32 - r as f32) / d as f32) + 2.0)) as u16
      } else {
        (60.0 * (((r as f32 - g as f32) / d as f32) + 4.0)) as u16
      };
      (h, 60, 40)
    } else {
      hash_to_hsl(hash)
    }
  } else {
    hash_to_hsl(hash)
  };

  let bg_style = format!("background: hsl({hue}, {sat}%, {light}%)");
  let extra_class = class.unwrap_or_default();

  rsx! {
    div {
      class: "relative flex items-center justify-center w-11 h-11 text-base font-semibold text-white overflow-hidden {extra_class}",
      style: "{bg_style}",
      span { class: "relative z-10 drop-shadow-[0_1px_2px_rgba(0,0,0,0.65)]", "{initial}" }
    }
  }
}
