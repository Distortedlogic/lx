use charming::{
    Chart,
    component::{Axis, DataZoom, DataZoomType, Legend, LegendSelectedMode},
    element::{
        AreaStyle, AxisType, Color, Emphasis, EmphasisFocus, ItemStyle, LineStyle, Orient,
        Tooltip, Trigger,
    },
    series::Line,
};
use dioxus::prelude::*;

use crate::charming_wrapper::CharmingChart;
use crate::empty_state::EmptyState;
use crate::expandable::ChartExpanded;
use crate::theme::NO_DATA_AVAILABLE;
use crate::types::{DataZoomConfig, LegendPosition, LineData, LineSeries};

#[component]
pub fn GenericLineChart(
    series: Vec<LineSeries>,
    #[props(default)] title: Option<&'static str>,
    #[props(default)] x_type: Option<AxisType>,
    #[props(default)] x_data: Option<Vec<String>>,
    #[props(default)] x_name: Option<String>,
    #[props(default)] y_name: Option<String>,
    #[props(default)] x_fmt: Option<&'static str>,
    #[props(default)] y_fmt: Option<&'static str>,
    #[props(default)] tooltip_fmt: Option<&'static str>,
    #[props(default)] datazoom: Option<DataZoomConfig>,
    #[props(default)] legend: bool,
    #[props(default)] legend_show: Option<bool>,
    #[props(default)] legend_position: Option<LegendPosition>,
    #[props(default)] y_min: Option<f64>,
    #[props(default)] boundary_gap: Option<bool>,
) -> Element {
    if series.iter().all(|s| s.data.is_empty()) {
        return rsx! { EmptyState { message: NO_DATA_AVAILABLE.to_owned() } };
    }
    let mut x_axis = Axis::new().type_(x_type.unwrap_or(AxisType::Value));
    if let Some(data) = &x_data {
        x_axis = x_axis.data(data.clone());
    }
    if let Some(name) = &x_name {
        x_axis = x_axis.name(name);
    }
    if let Some(gap) = boundary_gap {
        x_axis = x_axis.boundary_gap(gap);
    }
    let mut y_axis = Axis::new().type_(AxisType::Value);
    if let Some(name) = &y_name {
        y_axis = y_axis.name(name);
    }
    if let Some(min) = y_min {
        y_axis = y_axis.min(min);
    }
    let mut chart = Chart::new()
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .x_axis(x_axis)
        .y_axis(y_axis);
    let mut legend_data = Vec::new();
    for s in &series {
        let mut line = Line::new().name(&s.name);
        match &s.data {
            LineData::Pairs(pairs) => line = line.data(pairs.clone()),
            LineData::Values(vals) => line = line.data(vals.clone()),
        }
        line = line
            .item_style(ItemStyle::new().color(Color::from(s.color)))
            .line_style(
                LineStyle::new()
                    .color(Color::from(s.color))
                    .width(s.width.unwrap_or(2)),
            )
            .emphasis(Emphasis::new().focus(EmphasisFocus::Series))
            .show_symbol(s.show_symbol);
        if s.area {
            line = line.area_style(AreaStyle::new().opacity(0.7));
        }
        if let Some(stack) = &s.stack {
            line = line.stack(stack);
        }
        legend_data.push(s.name.clone());
        chart = chart.series(line);
    }
    if legend {
        let pos = legend_position.unwrap_or_default();
        let mut leg = Legend::new()
            .data(legend_data)
            .selected_mode(LegendSelectedMode::Multiple)
            .show(legend_show.unwrap_or(true));
        match pos {
            LegendPosition::Bottom => leg = leg.orient(Orient::Horizontal).bottom(0),
            LegendPosition::Right => leg = leg.orient(Orient::Vertical).right(0),
            LegendPosition::Top => {}
        }
        chart = chart.legend(leg);
    }
    let is_expanded = try_use_context::<ChartExpanded>().is_some_and(|c| c.0);
    if is_expanded {
        let zoom = datazoom.unwrap_or_default();
        if zoom.slider {
            chart = chart.data_zoom(
                DataZoom::new()
                    .type_(DataZoomType::Slider)
                    .start(zoom.start)
                    .end(zoom.end),
            );
        }
        if zoom.inside {
            chart = chart.data_zoom(
                DataZoom::new()
                    .type_(DataZoomType::Inside)
                    .start(zoom.start)
                    .end(zoom.end),
            );
        }
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
