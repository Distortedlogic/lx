pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod collapsible;
pub mod command;
pub mod dialog;
pub mod dropdown_menu;
pub mod input;
pub mod label;
pub mod popover;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod sheet;
pub mod skeleton;
pub mod tabs;
pub mod textarea;
pub mod tooltip;

pub fn cn(classes: &[&str]) -> String {
  classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
