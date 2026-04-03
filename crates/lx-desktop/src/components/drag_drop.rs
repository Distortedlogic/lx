use base64::Engine;
use dioxus::prelude::*;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct DroppedFile {
  pub name: String,
  pub mime_type: String,
  pub size: u64,
  pub data_base64: String,
}

fn save_dropped_file(file: &DroppedFile) -> Option<String> {
  let dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join("lx-uploads");
  std::fs::create_dir_all(&dir).ok()?;
  let safe_name = Path::new(&file.name).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "unnamed".to_string());
  let suffix = uuid::Uuid::new_v4().simple().to_string();
  let suffix_short = &suffix[..8];
  let unique_name = match safe_name.rsplit_once('.') {
    Some((stem, ext)) => format!("{stem}_{suffix_short}.{ext}"),
    None => format!("{safe_name}_{suffix_short}"),
  };
  let path = dir.join(unique_name);
  let bytes = base64::engine::general_purpose::STANDARD.decode(&file.data_base64).ok()?;
  std::fs::write(&path, bytes).ok()?;
  Some(path.to_string_lossy().into_owned())
}

pub fn install_drop_listener() {
  spawn(async {
    let js = r#"
          if (!window._dropListenerInstalled) {
            window._dropListenerInstalled = true;
            document.addEventListener('drop', function(e) {
              var files = e.dataTransfer ? e.dataTransfer.files : [];
              var captured = [];
              var pending = files.length;
              if (pending === 0) {
                window._lastDropFiles = '[]';
                return;
              }
              for (var i = 0; i < files.length; i++) {
                (function(file) {
                  var reader = new FileReader();
                  reader.onload = function() {
                    captured.push({
                      name: file.name,
                      mime: file.type || 'application/octet-stream',
                      size: file.size,
                      data: reader.result.split(',')[1] || ''
                    });
                    pending--;
                    if (pending === 0) {
                      window._lastDropFiles = JSON.stringify(captured);
                    }
                  };
                  reader.readAsDataURL(file);
                })(files[i]);
              }
            }, true);
            document.addEventListener('dragover', function(e) {
              e.preventDefault();
            }, true);
          }
        "#;
    let _ = document::eval(js).await;
  });
}

pub async fn read_dropped_files() -> Vec<DroppedFile> {
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  let js = r#"
      (function() {
        var data = window._lastDropFiles || '[]';
        window._lastDropFiles = '[]';
        return data;
      })()
    "#;
  let files: Vec<serde_json::Value> = match document::eval(js).await {
    Ok(result) => {
      let s = result.to_string();
      let s = s.trim_matches('"');
      let unescaped = s.replace("\\\"", "\"").replace("\\\\", "\\");
      serde_json::from_str(&unescaped).unwrap_or_default()
    },
    Err(_) => vec![],
  };
  files
    .iter()
    .filter_map(|f| {
      Some(DroppedFile {
        name: f["name"].as_str()?.to_string(),
        mime_type: f["mime"].as_str().unwrap_or("application/octet-stream").to_string(),
        size: f["size"].as_u64().unwrap_or(0),
        data_base64: f["data"].as_str().unwrap_or("").to_string(),
      })
    })
    .collect()
}

pub fn build_markdown_links(files: &[DroppedFile]) -> String {
  let mut links = String::new();
  for file in files {
    match save_dropped_file(file) {
      Some(path) => {
        if file.mime_type.starts_with("image/") {
          links.push_str(&format!("\n![{}]({})", file.name, path));
        } else {
          links.push_str(&format!("\n[{}]({})", file.name, path));
        }
      },
      None => {
        links.push_str(&format!("\n[{} (upload failed)]()", file.name));
      },
    }
  }
  links
}

#[component]
pub fn DragOverlay() -> Element {
  rsx! {
    div { class: "absolute inset-0 z-10 flex items-center justify-center bg-[var(--surface)]/80 pointer-events-none",
      div { class: "flex flex-col items-center gap-2 text-[var(--primary)]",
        span { class: "material-symbols-outlined text-xl", "upload_file" }
        span { class: "text-sm font-medium", "Drop files here" }
      }
    }
  }
}
