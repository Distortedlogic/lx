use charming::Chart;
use charming::component::Title;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn CharmingChart(
    chart: Chart,
    #[props(default)] title: Option<&'static str>,
    #[props(default)] x_fmt: Option<&'static str>,
    #[props(default)] y_fmt: Option<&'static str>,
    #[props(default)] tooltip_fmt: Option<&'static str>,
    #[props(default)] label_fmt: Option<&'static str>,
    #[props(default)] extra_data: Option<String>,
) -> Element {
    let chart_id = use_memo(|| format!("chart-{}", Uuid::new_v4().simple()));

    let mut chart_json_signal = use_signal(String::new);

    let chart = if let Some(t) = title {
        chart.title(Title::new().text(t))
    } else {
        chart
    };

    let current_json = chart.to_string();
    if current_json != *chart_json_signal.peek() {
        chart_json_signal.set(current_json);
    }

    use_effect(move || {
        let chart_json = chart_json_signal();
        if chart_json.is_empty() {
            return;
        }
        let id = chart_id.peek().clone();
        document::eval(&format!("LxCharts.initChart('{id}', {chart_json})"));
    });

    use_drop(move || {
        let id = chart_id();
        document::eval(&format!("LxCharts.disposeChart('{id}')"));
    });

    rsx! {
        div {
            id: "{chart_id}",
            class: "w-full h-full min-h-32",
            "data-x-fmt": x_fmt.unwrap_or_default(),
            "data-y-fmt": y_fmt.unwrap_or_default(),
            "data-tooltip-fmt": tooltip_fmt.unwrap_or_default(),
            "data-label-fmt": label_fmt.unwrap_or_default(),
            "data-extra": extra_data.as_deref().unwrap_or_default(),
        }
    }
}
