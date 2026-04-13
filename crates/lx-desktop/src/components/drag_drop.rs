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

fn save_dropped_file(file: &DroppedFile) -> Result<String, String> {
  let dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join("lx-uploads");
  std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
  let safe_name = Path::new(&file.name).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "unnamed".to_string());
  let suffix = uuid::Uuid::new_v4().simple().to_string();
  let suffix_short = &suffix[..8];
  let unique_name = match safe_name.rsplit_once('.') {
    Some((stem, ext)) => format!("{stem}_{suffix_short}.{ext}"),
    None => format!("{safe_name}_{suffix_short}"),
  };
  let path = dir.join(unique_name);
  let bytes = base64::engine::general_purpose::STANDARD.decode(&file.data_base64).map_err(|e| format!("cannot decode dropped file '{}': {e}", file.name))?;
  std::fs::write(&path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
  Ok(path.to_string_lossy().into_owned())
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

pub async fn read_dropped_files() -> Result<Vec<DroppedFile>, String> {
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  let js = r#"
      (function() {
        var data = window._lastDropFiles || '[]';
        window._lastDropFiles = '[]';
        return data;
      })()
    "#;
  let result = document::eval(js).await.map_err(|e| format!("browser drop payload unavailable: {e}"))?;
  let s = result.to_string();
  let s = s.trim_matches('"');
  let unescaped = s.replace("\\\"", "\"").replace("\\\\", "\\");
  let files: Vec<serde_json::Value> = serde_json::from_str(&unescaped).map_err(|e| format!("invalid browser drop payload: {e}"))?;
  let mut dropped = Vec::with_capacity(files.len());
  for (idx, file) in files.iter().enumerate() {
    let name = file.get("name").and_then(|v| v.as_str()).ok_or_else(|| format!("invalid dropped file payload at index {idx}: missing name"))?;
    let data_base64 = file.get("data").and_then(|v| v.as_str()).ok_or_else(|| format!("invalid dropped file payload at index {idx}: missing data"))?;
    dropped.push(DroppedFile {
      name: name.to_string(),
      mime_type: file.get("mime").and_then(|v| v.as_str()).unwrap_or("application/octet-stream").to_string(),
      size: file.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
      data_base64: data_base64.to_string(),
    });
  }
  Ok(dropped)
}

pub fn build_markdown_links(files: &[DroppedFile]) -> String {
  let mut links = String::new();
  for file in files {
    match save_dropped_file(file) {
      Ok(path) => {
        if file.mime_type.starts_with("image/") {
          links.push_str(&format!("\n![{}]({})", file.name, path));
        } else {
          links.push_str(&format!("\n[{}]({})", file.name, path));
        }
      },
      Err(e) => {
        links.push_str(&format!("\n[{} (upload failed: {})]()", file.name, e));
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn upload_failure_placeholder_includes_reason() {
    let file = DroppedFile { name: "bad.txt".to_string(), mime_type: "text/plain".to_string(), size: 1, data_base64: "!!!not-base64!!!".to_string() };

    let error = save_dropped_file(&file).expect_err("invalid base64 should fail");
    let markdown = build_markdown_links(&[file]);

    assert!(markdown.contains("upload failed:"));
    assert!(markdown.contains(&error));
  }
}
