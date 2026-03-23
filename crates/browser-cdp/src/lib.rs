use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams,
    DispatchMouseEventType, MouseButton,
};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, GetNavigationHistoryParams, NavigateToHistoryEntryParams,
};
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::browser::HeadlessMode;
use chromiumoxide::{Browser, BrowserConfig, Handler, Page};
use dashmap::DashMap;
use futures::StreamExt;
use tokio::sync::{Mutex, OnceCell};

static BROWSER_INSTANCE: OnceCell<Arc<Mutex<Browser>>> = OnceCell::const_new();
static SESSIONS: LazyLock<DashMap<String, Arc<BrowserSession>>> =
    LazyLock::new(DashMap::new);

async fn get_browser() -> Result<Arc<Mutex<Browser>>> {
    let browser = BROWSER_INSTANCE
        .get_or_try_init(|| async {
            let config = BrowserConfig::builder()
                .no_sandbox()
                .headless_mode(HeadlessMode::New)
                .arg("--disable-gpu")
                .arg("--disable-dev-shm-usage")
                .build()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let (browser, mut handler): (Browser, Handler) = Browser::launch(config)
                .await
                .context("failed to launch browser")?;

            tokio::spawn(async move {
                while handler.next().await.is_some() {}
            });

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

    pub async fn screenshot(&self) -> Result<String> {
        let page = self.page.lock().await;
        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Jpeg)
            .quality(70)
            .build();
        let bytes = page.screenshot(params).await.context("screenshot failed")?;
        Ok(B64.encode(&bytes))
    }

    pub async fn go_back(&self) -> Result<()> {
        let page = self.page.lock().await;
        let history = page
            .execute(GetNavigationHistoryParams {})
            .await
            .context("get navigation history failed")?;
        let idx = history.result.current_index;
        if idx > 0 {
            if let Some(entry) = history.result.entries.get((idx - 1) as usize) {
                page.execute(NavigateToHistoryEntryParams::new(entry.id))
                    .await
                    .context("go_back failed")?;
            }
        }
        Ok(())
    }

    pub async fn go_forward(&self) -> Result<()> {
        let page = self.page.lock().await;
        let history = page
            .execute(GetNavigationHistoryParams {})
            .await
            .context("get navigation history failed")?;
        let idx = history.result.current_index as usize;
        if idx + 1 < history.result.entries.len() {
            if let Some(entry) = history.result.entries.get(idx + 1) {
                page.execute(NavigateToHistoryEntryParams::new(entry.id))
                    .await
                    .context("go_forward failed")?;
            }
        }
        Ok(())
    }

    pub async fn reload(&self) -> Result<()> {
        let page = self.page.lock().await;
        page.reload().await.context("reload failed")?;
        Ok(())
    }
}

pub async fn get_or_create_session(id: &str) -> Result<Arc<BrowserSession>> {
    if let Some(session) = SESSIONS.get(id) {
        return Ok(Arc::clone(session.value()));
    }

    let browser_arc = get_browser().await?;
    let browser = browser_arc.lock().await;
    let page = browser
        .new_page("about:blank")
        .await
        .context("failed to create new page")?;

    let session = Arc::new(BrowserSession {
        page: Mutex::new(page),
    });
    SESSIONS.insert(id.to_owned(), Arc::clone(&session));
    Ok(session)
}

pub fn remove_session(id: &str) {
    SESSIONS.remove(id);
}
