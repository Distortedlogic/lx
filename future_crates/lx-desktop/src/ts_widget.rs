use dioxus::prelude::*;
use serde::Serialize;

pub fn use_ts_widget(widget: &str, config: impl Serialize) -> (String, TsWidgetHandle) {
  let element_id = use_hook(|| format!("tw-{}", uuid::Uuid::new_v4()));
  let mut eval: Signal<Option<document::Eval>> = use_signal(|| None);

  let eid = element_id.clone();
  let init_msg = serde_json::json!({
      "widget": widget,
      "elementId": eid,
      "config": serde_json::to_value(&config).unwrap_or_default(),
  });

  use_future(move || {
    let init_msg = init_msg.clone();
    async move {
      let e = document::eval("await LxDesktop.runWidgetBridge(dioxus)");
      let _ = e.send(init_msg);
      eval.set(Some(e));
    }
  });

  let eid_drop = element_id.clone();
  use_drop(move || {
    if let Some(e) = &*eval.peek() {
      let _ = e.send(serde_json::json!({ "action": "dispose" }));
    }
    let _ = eid_drop;
  });

  (element_id, TsWidgetHandle { eval })
}

#[derive(Clone, Copy)]
pub struct TsWidgetHandle {
  eval: Signal<Option<document::Eval>>,
}

impl TsWidgetHandle {
  pub fn send_update(&self, data: impl Serialize) {
    if let Some(e) = &*self.eval.peek() {
      let _ = e.send(serde_json::json!({
          "action": "update",
          "data": serde_json::to_value(&data).unwrap_or_default(),
      }));
    }
  }

  pub fn send_resize(&self) {
    if let Some(e) = &*self.eval.peek() {
      let _ = e.send(serde_json::json!({ "action": "resize" }));
    }
  }

  pub async fn recv<T: serde::de::DeserializeOwned>(&self) -> Result<T, document::EvalError> {
    let Some(mut e) = *self.eval.peek() else {
      return Err(document::EvalError::Finished);
    };
    e.recv::<T>().await
  }
}
