use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
  pub left: f64,
  pub top: f64,
  pub width: f64,
  pub height: f64,
}

impl Default for Rect {
  fn default() -> Self {
    Self { left: 0.0, top: 0.0, width: 100.0, height: 100.0 }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DividerInfo {
  pub rect: Rect,
  pub parent_rect: Rect,
  pub direction: SplitDirection,
  pub split_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
  Horizontal,
  Vertical,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaneNode<L: Clone> {
  Leaf(L),
  Split { id: String, direction: SplitDirection, ratio: f64, first: Box<Self>, second: Box<Self> },
}

pub trait Pane: Clone {
  fn pane_id(&self) -> &str;
}

impl<L: Pane> PaneNode<L> {
  pub fn pane_id(&self) -> Option<&str> {
    match self {
      Self::Leaf(leaf) => Some(leaf.pane_id()),
      Self::Split { .. } => None,
    }
  }

  pub fn leaf(&self) -> Option<&L> {
    match self {
      Self::Leaf(leaf) => Some(leaf),
      Self::Split { .. } => None,
    }
  }

  pub fn split(self, target_id: &str, direction: SplitDirection, new_pane: Self) -> Self {
    if self.pane_id().is_some_and(|id| id == target_id) {
      return Self::Split { id: Uuid::new_v4().to_string(), direction, ratio: 0.5, first: Box::new(self), second: Box::new(new_pane) };
    }
    match self {
      Self::Split { id, direction: d, ratio, first, second } => Self::Split {
        id,
        direction: d,
        ratio,
        first: Box::new(first.split(target_id, direction, new_pane.clone())),
        second: Box::new(second.split(target_id, direction, new_pane)),
      },
      other => other,
    }
  }

  pub fn close(self, target_id: &str) -> Option<Self> {
    if self.pane_id().is_some_and(|id| id == target_id) {
      return None;
    }
    match self {
      Self::Leaf(_) => Some(self),
      Self::Split { id, direction, ratio, first, second } => {
        if first.pane_id().is_some_and(|id| id == target_id) {
          return Some(*second);
        }
        if second.pane_id().is_some_and(|id| id == target_id) {
          return Some(*first);
        }
        match (first.close(target_id), second.close(target_id)) {
          (Some(f), Some(s)) => Some(Self::Split { id, direction, ratio, first: Box::new(f), second: Box::new(s) }),
          (Some(f), None) => Some(f),
          (None, Some(s)) => Some(s),
          (None, None) => None,
        }
      },
    }
  }

  pub fn convert(self, target_id: &str, replacement: Self) -> Self {
    if self.pane_id().is_some_and(|id| id == target_id) {
      return replacement;
    }
    match self {
      Self::Split { id, direction, ratio, first, second } => Self::Split {
        id,
        direction,
        ratio,
        first: Box::new(first.convert(target_id, replacement.clone())),
        second: Box::new(second.convert(target_id, replacement)),
      },
      other => other,
    }
  }

  pub fn set_ratio_by_split_id(self, split_id: &str, new_ratio: f64) -> Self {
    match self {
      Self::Split { id, direction, ratio, first, second } => {
        if id == split_id {
          Self::Split { id, direction, ratio: new_ratio.clamp(0.1, 0.9), first, second }
        } else {
          Self::Split {
            id,
            direction,
            ratio,
            first: Box::new(first.set_ratio_by_split_id(split_id, new_ratio)),
            second: Box::new(second.set_ratio_by_split_id(split_id, new_ratio)),
          }
        }
      },
      other => other,
    }
  }

  pub fn all_pane_ids(&self) -> Vec<String> {
    match self {
      Self::Leaf(leaf) => vec![leaf.pane_id().to_owned()],
      Self::Split { first, second, .. } => {
        let mut ids = first.all_pane_ids();
        ids.extend(second.all_pane_ids());
        ids
      },
    }
  }

  pub fn first_leaf(&self) -> Option<&L> {
    match self {
      Self::Leaf(leaf) => Some(leaf),
      Self::Split { first, second, .. } => first.first_leaf().or_else(|| second.first_leaf()),
    }
  }

  pub fn find_leaf(&self, target_id: &str) -> Option<&L> {
    match self {
      Self::Leaf(leaf) if leaf.pane_id() == target_id => Some(leaf),
      Self::Leaf(_) => None,
      Self::Split { first, second, .. } => first.find_leaf(target_id).or_else(|| second.find_leaf(target_id)),
    }
  }
}
