use crate::charming_wrapper::CharmingChart;
use crate::empty_state::EmptyState;
use crate::theme::NO_DATA_AVAILABLE;
use crate::types::PieSlice;
use charming::{
    Chart,
    datatype::DataPointItem,
    element::{Color, Formatter, ItemStyle, Label, Tooltip, Trigger},
    series::Pie,
};
use dioxus::prelude::*;

#[component]
pub fn GenericPieChart(
    data: Vec<PieSlice>,
    #[props(default)] title: Option<&'static str>,
    #[props(default)] radius: Option<(String, String)>,
    #[props(default)] tooltip_fmt: Option<&'static str>,
    #[props(default)] label_format: Option<String>,
) -> Element {
    if data.is_empty() {
        return rsx! {
          EmptyState { message: NO_DATA_AVAILABLE.to_owned() }
        };
    }
    let items: Vec<DataPointItem> = data
        .iter()
        .map(|s| {
            let mut item = DataPointItem::new(s.value).name(&s.name);
            if let Some(color) = s.color {
                item = item.item_style(ItemStyle::new().color(Color::from(color)));
            }
            item
        })
        .collect();
    let r = radius.unwrap_or_else(|| ("40%".to_owned(), "60%".to_owned()));
    let fmt = label_format.as_deref().unwrap_or("{b}: {c} ({d}%)");
    let chart = Chart::new()
        .tooltip(
            Tooltip::new()
                .trigger(Trigger::Item)
                .formatter(Formatter::String(fmt.to_owned())),
        )
        .series(
            Pie::new()
                .radius(vec![r.0, r.1])
                .data(items)
                .avoid_label_overlap(true)
                .label(Label::new().show(true).formatter(fmt)),
        );
    rsx! {
      CharmingChart {
        chart,
        title,
        tooltip_fmt: tooltip_fmt.unwrap_or_default(),
      }
    }
}
