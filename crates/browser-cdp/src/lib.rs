use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result};
use chromiumoxide::cdp::browser_protocol::input::{
  DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use chromiumoxide::cdp::browser_protocol::network::DisableParams as NetworkDisableParams;
use chromiumoxide::cdp::browser_protocol::page::{
  EventScreencastFrame, GetNavigationHistoryParams, NavigateToHistoryEntryParams, ScreencastFrameAckParams, StartScreencastFormat, StartScreencastParams,
};
use chromiumoxide::listeners::EventStream;
use chromiumoxide::{Browser, BrowserConfig, Handler, Page};
use dashmap::DashMap;
use futures::StreamExt;
use tokio::sync::{Mutex, OnceCell};

static BROWSER_INSTANCE: OnceCell<Arc<Mutex<Browser>>> = OnceCell::const_new();
static SESSIONS: LazyLock<DashMap<String, Arc<BrowserSession>>> = LazyLock::new(DashMap::new);

async fn get_browser() -> Result<Arc<Mutex<Browser>>> {
  let browser = BROWSER_INSTANCE
    .get_or_try_init(|| async {
      let user_data_dir = std::env::temp_dir().join(format!("lx-cdp-{}", std::process::id()));
      let config = BrowserConfig::builder()
        .no_sandbox()
        .new_headless_mode()
        .arg("--disable-gpu")
        .arg("--disable-dev-shm-usage")
        .user_data_dir(&user_data_dir)
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

      let (browser, mut handler): (Browser, Handler) = Browser::launch(config).await.context("failed to launch browser")?;

      tokio::spawn(async move { while handler.next().await.is_some() {} });

      Ok::<_, anyhow::Error>(Arc::new(Mutex::new(browser)))
    })
    .await?;
  Ok(Arc::clone(browser))
}

pub struct BrowserSession {
  page: Mutex<Page>,
}

impl BrowserSession {
  pub async fn navigate(&self, url: &str) -> Result<(String, String)> {
    let page = self.page.lock().await;
    page.goto(url).await.context("navigation failed")?;
    let final_url = page.url().await?.unwrap_or_default();
    let title = page.get_title().await?.unwrap_or_default();
    Ok((final_url, title))
  }

  pub async fn click(&self, x: f64, y: f64) -> Result<()> {
    let page = self.page.lock().await;
    let mut pressed = DispatchMouseEventParams::new(DispatchMouseEventType::MousePressed, x, y);
    pressed.button = Some(MouseButton::Left);
    pressed.click_count = Some(1);
    page.execute(pressed).await.context("click pressed failed")?;
    let mut released = DispatchMouseEventParams::new(DispatchMouseEventType::MouseReleased, x, y);
    released.button = Some(MouseButton::Left);
    released.click_count = Some(1);
    page.execute(released).await.context("click released failed")?;
    Ok(())
  }

  pub async fn type_text(&self, text: &str) -> Result<()> {
    let page = self.page.lock().await;
    for c in text.chars() {
      let mut params = DispatchKeyEventParams::new(DispatchKeyEventType::Char);
      let s = c.to_string();
      params.text = Some(s.clone());
      params.unmodified_text = Some(s);
      page.execute(params).await.context("type_text failed")?;
    }
    Ok(())
  }

  pub async fn dispatch_key(&self, key: &str, code: &str, modifiers: i64) -> Result<()> {
    let page = self.page.lock().await;
    let is_printable = key.len() == 1 && modifiers & !8 == 0;

    let mut down = DispatchKeyEventParams::new(DispatchKeyEventType::KeyDown);
    down.key = Some(key.to_owned());
    down.code = Some(code.to_owned());
    if modifiers != 0 {
      down.modifiers = Some(modifiers);
    }
    if is_printable {
      down.text = Some(key.to_owned());
    }
    page.execute(down).await.context("dispatch_key KeyDown failed")?;

    if is_printable {
      let mut char_evt = DispatchKeyEventParams::new(DispatchKeyEventType::Char);
      char_evt.key = Some(key.to_owned());
      char_evt.code = Some(code.to_owned());
      char_evt.text = Some(key.to_owned());
      char_evt.unmodified_text = Some(key.to_owned());
      if modifiers != 0 {
        char_evt.modifiers = Some(modifiers);
      }
      page.execute(char_evt).await.context("dispatch_key Char failed")?;
    }

    let mut up = DispatchKeyEventParams::new(DispatchKeyEventType::KeyUp);
    up.key = Some(key.to_owned());
    up.code = Some(code.to_owned());
    if modifiers != 0 {
      up.modifiers = Some(modifiers);
    }
    page.execute(up).await.context("dispatch_key KeyUp failed")?;
    Ok(())
  }

  pub async fn scroll(&self, x: f64, y: f64, delta_x: f64, delta_y: f64) -> Result<()> {
    let page = self.page.lock().await;
    let mut params = DispatchMouseEventParams::new(DispatchMouseEventType::MouseWheel, x, y);
    params.delta_x = Some(delta_x);
    params.delta_y = Some(delta_y);
    page.execute(params).await.context("scroll failed")?;
    Ok(())
  }

  pub async fn start_screencast(&self) -> Result<EventStream<EventScreencastFrame>> {
    let page = self.page.lock().await;
    let stream = page.event_listener::<EventScreencastFrame>().await.context("screencast listener failed")?;
    let params = StartScreencastParams::builder().format(StartScreencastFormat::Jpeg).quality(70).every_nth_frame(1).build();
    page.execute(params).await.context("start screencast failed")?;
    Ok(stream)
  }

  pub async fn ack_frame(&self, session_id: i64) -> Result<()> {
    let page = self.page.lock().await;
    page.execute(ScreencastFrameAckParams::new(session_id)).await.context("screencast ack failed")?;
    Ok(())
  }

  pub async fn go_back(&self) -> Result<()> {
    let page = self.page.lock().await;
    let history = page.execute(GetNavigationHistoryParams {}).await.context("get navigation history failed")?;
    let idx = history.result.current_index;
    if idx > 0
      && let Some(entry) = history.result.entries.get((idx - 1) as usize)
    {
      page.execute(NavigateToHistoryEntryParams::new(entry.id)).await.context("go_back failed")?;
    }
    Ok(())
  }

  pub async fn go_forward(&self) -> Result<()> {
    let page = self.page.lock().await;
    let history = page.execute(GetNavigationHistoryParams {}).await.context("get navigation history failed")?;
    let idx = history.result.current_index as usize;
    if idx + 1 < history.result.entries.len()
      && let Some(entry) = history.result.entries.get(idx + 1)
    {
      page.execute(NavigateToHistoryEntryParams::new(entry.id)).await.context("go_forward failed")?;
    }
    Ok(())
  }

  pub async fn reload(&self) -> Result<()> {
    let page = self.page.lock().await;
    page.reload().await.context("reload failed")?;
    Ok(())
  }

  pub async fn current_url(&self) -> Result<String> {
    let page = self.page.lock().await;
    Ok(page.url().await?.unwrap_or_default())
  }
}

pub async fn get_or_create_session(id: &str) -> Result<Arc<BrowserSession>> {
  if let Some(session) = SESSIONS.get(id) {
    return Ok(Arc::clone(session.value()));
  }

  let browser_arc = get_browser().await?;
  let browser = browser_arc.lock().await;
  let page = browser.new_page("about:blank").await.context("failed to create new page")?;
  page.execute(NetworkDisableParams {}).await.context("failed to disable network events")?;

  let session = Arc::new(BrowserSession { page: Mutex::new(page) });
  SESSIONS.insert(id.to_owned(), Arc::clone(&session));
  Ok(session)
}

pub fn remove_session(id: &str) {
  SESSIONS.remove(id);
}
