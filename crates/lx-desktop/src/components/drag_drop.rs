use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DroppedFile {
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub data_base64: String,
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
        }
        Err(_) => vec![],
    };
    files
        .iter()
        .filter_map(|f| {
            Some(DroppedFile {
                name: f["name"].as_str()?.to_string(),
                mime_type: f["mime"]
                    .as_str()
                    .unwrap_or("application/octet-stream")
                    .to_string(),
                size: f["size"].as_u64().unwrap_or(0),
                data_base64: f["data"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect()
}

pub fn build_markdown_links(files: &[DroppedFile]) -> String {
    let mut links = String::new();
    for file in files {
        if file.mime_type.starts_with("image/") {
            links.push_str(&format!("\n![{}](upload://{})", file.name, file.name));
        } else {
            links.push_str(&format!("\n[{}](upload://{})", file.name, file.name));
        }
    }
    links
}

#[component]
pub fn DragOverlay() -> Element {
    rsx! {
        div {
            class: "absolute inset-0 z-10 flex items-center justify-center bg-[var(--surface)]/80 pointer-events-none",
            div { class: "flex flex-col items-center gap-2 text-[var(--primary)]",
                span { class: "material-symbols-outlined text-3xl", "upload_file" }
                span { class: "text-sm font-medium", "Drop files here" }
            }
        }
    }
}
