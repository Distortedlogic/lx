use dioxus::prelude::*;
use dioxus_hooks::use_future;
use tracing::info;

static APP_STATE: LazyLock<Mutex<AppState>> = LazyLock::new(|| Mutex::new(AppState::default()));

struct AppState {
    count: i32,
    name: String,
}

#[server]
async fn get_data() -> Result<Vec<String>, ServerFnError> {
    Err(ServerFnError::new("not implemented"))
}

#[component]
fn Counter(initial: i32) -> Element {
    let mut count = use_signal(|| initial);
    let data = use_resource(move || async { get_data().await });

    let action = use_action(move |_: ()| async move { get_data().await });
    use_effect(move || {
        if action.value().is_some() {
            info!("action complete");
        }
    });

    let status = use_memo(move || app_store.name());

    rsx! {
        div {
            class: "{nav_width} bg-card border-r border-border flex flex-col",
            button { onclick: move |_| count += 1, "Count: {count}" }
        }
    }
}

#[component]
fn Wrapper(children: Element) -> Element {
    rsx! { div { class: "p-4", {children} } }
}

fn format_item(store: &AppState, key: &str) -> String {
    format!("{}: {}", key, store.name)
}
