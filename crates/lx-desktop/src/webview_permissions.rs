use dioxus::desktop::wry::WebViewExtUnix;
use webkit2gtk::glib::prelude::ObjectExt;
use webkit2gtk::{PermissionRequestExt, SettingsExt, UserMediaPermissionRequest, WebViewExt};

pub fn enable_media_permissions(desktop: &dioxus::desktop::DesktopContext) {
  let wk_webview = desktop.webview.webview();

  if let Some(settings) = WebViewExt::settings(&wk_webview) {
    settings.set_enable_media_stream(true);
    settings.set_media_playback_requires_user_gesture(false);
  }

  wk_webview.connect_permission_request(|_webview, request: &webkit2gtk::PermissionRequest| {
    if request.is::<UserMediaPermissionRequest>() {
      request.allow();
      return true;
    }
    false
  });
}
