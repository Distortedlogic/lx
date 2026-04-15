pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod collapsible;
pub mod command;
pub mod dialog;
pub mod dropdown_menu;
pub mod popover;
pub mod scroll_area;
pub mod select;
pub mod sheet;
pub mod tabs;

pub fn cn(classes: &[&str]) -> String {
  classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
