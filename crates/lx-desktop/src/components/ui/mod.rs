pub mod avatar;
pub mod badge;
pub mod button;
pub mod checkbox;
pub mod input;
pub mod label;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod textarea;

pub fn cn(classes: &[&str]) -> String {
  classes.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}
