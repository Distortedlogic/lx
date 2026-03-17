use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AiCallProps {
    pub call_id: u64,
    pub model: String,
    pub prompt: String,
    pub response: Option<String>,
    pub cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[component]
pub fn AiCallWidget(props: AiCallProps) -> Element {
    let cost_display = props
        .cost_usd
        .map(|c| format!("${c:.4}"))
        .unwrap_or_default();
    let duration_display = props
        .duration_ms
        .map(|d| format!("{d}ms"))
        .unwrap_or_default();

    rsx! {
        div {
            class: "ai-call",
            div {
                class: "ai-call-header",
                span { class: "ai-model", "[AI] {props.model}" }
                if !cost_display.is_empty() {
                    span { class: "ai-cost", "{cost_display}" }
                }
                if !duration_display.is_empty() {
                    span { class: "ai-duration", "{duration_display}" }
                }
            }
            div {
                class: "ai-call-prompt",
                h4 { "Prompt" }
                pre { "{props.prompt}" }
            }
            if let Some(ref resp) = props.response {
                div {
                    class: "ai-call-response",
                    h4 { "Response" }
                    pre { "{resp}" }
                }
            }
            if let Some(ref err) = props.error {
                div {
                    class: "ai-call-error",
                    h4 { "Error" }
                    pre { class: "error-text", "{err}" }
                }
            }
        }
    }
}
