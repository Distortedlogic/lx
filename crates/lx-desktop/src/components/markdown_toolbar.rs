use dioxus::prelude::*;

pub fn insert_at_cursor(editor_id: &str, value: &str, before: &str, after: &str, on_change: EventHandler<String>) {
  let val = value.to_string();
  let editor_id = editor_id.to_string();
  let before = before.to_string();
  let after = after.to_string();
  spawn(async move {
    let js = format!(
      "(function() {{ var el = document.getElementById('{editor_id}'); if (!el) return JSON.stringify({{start: -1, end: -1}}); return JSON.stringify({{start: el.selectionStart, end: el.selectionEnd}}); }})()"
    );
    let (start, end) = match document::eval(&js).await {
      Ok(result) => {
        let s = result.to_string();
        let s = s.trim_matches('"');
        match serde_json::from_str::<serde_json::Value>(s) {
          Ok(v) => {
            let st = v["start"].as_i64().unwrap_or(-1);
            let en = v["end"].as_i64().unwrap_or(-1);
            if st >= 0 && en >= 0 { (st as usize, en as usize) } else { (val.len(), val.len()) }
          },
          Err(_) => (val.len(), val.len()),
        }
      },
      Err(_) => (val.len(), val.len()),
    };

    let start = start.min(val.len());
    let end = end.min(val.len());
    let selected = &val[start..end];
    let new_value = format!("{}{}{}{}{}", &val[..start], before, selected, after, &val[end..]);
    on_change.call(new_value);

    let new_cursor = start + before.len() + selected.len();
    let set_cursor_js = format!(
      "setTimeout(function() {{ var el = document.getElementById('{editor_id}'); if (el) {{ el.selectionStart = {new_cursor}; el.selectionEnd = {new_cursor}; el.focus(); }} }}, 0)"
    );
    let _ = document::eval(&set_cursor_js).await;
  });
}

#[component]
pub fn ToolbarButtons(editor_id: String, value: String, on_change: EventHandler<String>) -> Element {
  rsx! {
    div { class: "flex gap-0.5",
      ToolbarBtn {
        icon: "format_bold",
        on_click: {
            let eid = editor_id.clone();
            let v = value.clone();
            move |_| insert_at_cursor(&eid, &v, "**", "**", on_change)
        },
      }
      ToolbarBtn {
        icon: "format_italic",
        on_click: {
            let eid = editor_id.clone();
            let v = value.clone();
            move |_| insert_at_cursor(&eid, &v, "*", "*", on_change)
        },
      }
      ToolbarBtn {
        icon: "code",
        on_click: {
            let eid = editor_id.clone();
            let v = value.clone();
            move |_| insert_at_cursor(&eid, &v, "\n```\n", "\n```", on_change)
        },
      }
      ToolbarBtn {
        icon: "link",
        on_click: {
            let eid = editor_id.clone();
            let v = value.clone();
            move |_| insert_at_cursor(&eid, &v, "[", "](url)", on_change)
        },
      }
      ToolbarBtn {
        icon: "title",
        on_click: {
            let eid = editor_id.clone();
            let v = value.clone();
            move |_| insert_at_cursor(&eid, &v, "\n## ", "", on_change)
        },
      }
    }
  }
}

#[component]
fn ToolbarBtn(icon: &'static str, on_click: EventHandler<()>) -> Element {
  rsx! {
    button {
      class: "w-6 h-6 flex items-center justify-center text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] rounded transition-colors",
      onclick: move |_| on_click.call(()),
      span { class: "material-symbols-outlined text-sm", "{icon}" }
    }
  }
}
