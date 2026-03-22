use charming::{
    Chart,
    component::Axis,
    element::{AxisLabel, AxisType, Color, ItemStyle, Tooltip, Trigger},
    series::Bar,
};
use dioxus::prelude::*;

use crate::charming_wrapper::CharmingChart;
use crate::empty_state::EmptyState;
use crate::theme::NO_DATA_AVAILABLE;
use crate::types::BarSeries;

#[component]
pub fn GenericBarChart(
    series: Vec<BarSeries>,
    categories: Vec<String>,
    #[props(default)] title: Option<&'static str>,
    #[props(default)] x_name: Option<String>,
    #[props(default)] y_name: Option<String>,
    #[props(default)] x_fmt: Option<&'static str>,
    #[props(default)] y_fmt: Option<&'static str>,
    #[props(default)] tooltip_fmt: Option<&'static str>,
    #[props(default)] rotate_labels: Option<f64>,
) -> Element {
    if categories.is_empty() && series.iter().all(|s| s.data.is_empty()) {
        return rsx! { EmptyState { message: NO_DATA_AVAILABLE.to_owned() } };
    }
    let mut x_axis = Axis::new().type_(AxisType::Category).data(categories);
    if let Some(name) = &x_name {
        x_axis = x_axis.name(name);
    }
    if let Some(rotate) = rotate_labels {
        x_axis = x_axis.axis_label(AxisLabel::new().rotate(rotate));
    }
    let mut y_axis = Axis::new().type_(AxisType::Value);
    if let Some(name) = &y_name {
        y_axis = y_axis.name(name);
    }
    let mut chart = Chart::new()
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .x_axis(x_axis)
        .y_axis(y_axis);
    for s in &series {
        let mut bar = Bar::new().name(&s.name).data(s.data.clone());
        if let Some(color) = s.color {
            bar = bar.item_style(ItemStyle::new().color(Color::from(color)));
        }
        if let Some(stack) = &s.stack {
            bar = bar.stack(stack);
        }
        chart = chart.series(bar);
    }
    rsx! {
        CharmingChart {
            chart,
            title,
            x_fmt: x_fmt.unwrap_or_default(),
            y_fmt: y_fmt.unwrap_or_default(),
            tooltip_fmt: tooltip_fmt.unwrap_or_default(),
        }
    }
}
