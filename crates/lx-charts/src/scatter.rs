use crate::charming_wrapper::CharmingChart;
use crate::empty_state::EmptyState;
use crate::theme::NO_DATA_AVAILABLE;
use crate::types::ScatterSeries;
use charming::{
    Chart,
    component::Axis,
    element::{AxisType, Color, ItemStyle, Tooltip, Trigger},
    series::Scatter,
};
use dioxus::prelude::*;

#[component]
pub fn GenericScatterChart(
    series: Vec<ScatterSeries>,
    #[props(default)] title: Option<&'static str>,
    #[props(default)] x_name: Option<String>,
    #[props(default)] y_name: Option<String>,
    #[props(default)] x_fmt: Option<&'static str>,
    #[props(default)] y_fmt: Option<&'static str>,
    #[props(default)] tooltip_fmt: Option<&'static str>,
    #[props(default)] extra_data: Option<String>,
) -> Element {
    if series.iter().all(|s| s.data.is_empty()) {
        return rsx! {
          EmptyState { message: NO_DATA_AVAILABLE.to_owned() }
        };
    }
    let mut x_axis = Axis::new().type_(AxisType::Value);
    if let Some(name) = &x_name {
        x_axis = x_axis.name(name);
    }
    let mut y_axis = Axis::new().type_(AxisType::Value);
    if let Some(name) = &y_name {
        y_axis = y_axis.name(name);
    }
    let mut chart = Chart::new()
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .x_axis(x_axis)
        .y_axis(y_axis);
    for s in &series {
        let mut scatter = Scatter::new()
            .name(&s.name)
            .data(s.data.clone())
            .item_style(ItemStyle::new().color(Color::from(s.color)));
        if let Some(size) = s.symbol_size {
            scatter = scatter.symbol_size(size);
        }
        chart = chart.series(scatter);
    }
    rsx! {
      CharmingChart {
        chart,
        title,
        x_fmt: x_fmt.unwrap_or_default(),
        y_fmt: y_fmt.unwrap_or_default(),
        tooltip_fmt: tooltip_fmt.unwrap_or_default(),
        extra_data,
      }
    }
}
