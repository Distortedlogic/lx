pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod command;
pub mod select;

pub fn cn(classes: &[&str]) -> String {
  classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
