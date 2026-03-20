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
        Self {
            left: 0.0,
            top: 0.0,
            width: 100.0,
            height: 100.0,
        }
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
pub enum PaneNode {
    Terminal {
        id: String,
        working_dir: String,
        command: Option<String>,
    },
    Browser {
        id: String,
        url: String,
        devtools: bool,
    },
    Editor {
        id: String,
        file_path: String,
        language: Option<String>,
    },
    Agent {
        id: String,
        session_id: String,
        model: String,
    },
    Canvas {
        id: String,
        widget_type: String,
        config: serde_json::Value,
    },
    Split {
        id: String,
        direction: SplitDirection,
        ratio: f64,
        first: Box<Self>,
        second: Box<Self>,
    },
}

const DIVIDER_SIZE_PCT: f64 = 0.4;

impl PaneNode {
    pub fn pane_id(&self) -> Option<&str> {
        match self {
            Self::Terminal { id, .. }
            | Self::Browser { id, .. }
            | Self::Editor { id, .. }
            | Self::Agent { id, .. }
            | Self::Canvas { id, .. } => Some(id),
            Self::Split { .. } => None,
        }
    }

    pub fn split(self, target_id: &str, direction: SplitDirection, new_pane: Self) -> Self {
        if self.pane_id().is_some_and(|id| id == target_id) {
            return Self::Split {
                id: Uuid::new_v4().to_string(),
                direction,
                ratio: 0.5,
                first: Box::new(self),
                second: Box::new(new_pane),
            };
        }
        match self {
            Self::Split {
                id: sid,
                direction: d,
                ratio,
                first,
                second,
            } => Self::Split {
                id: sid,
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
            Self::Terminal { .. }
            | Self::Browser { .. }
            | Self::Editor { .. }
            | Self::Agent { .. }
            | Self::Canvas { .. } => Some(self),
            Self::Split {
                id: sid,
                direction,
                ratio,
                first,
                second,
            } => {
                if first.pane_id().is_some_and(|id| id == target_id) {
                    return Some(*second);
                }
                if second.pane_id().is_some_and(|id| id == target_id) {
                    return Some(*first);
                }
                match (first.close(target_id), second.close(target_id)) {
                    (Some(f), Some(s)) => Some(Self::Split {
                        id: sid,
                        direction,
                        ratio,
                        first: Box::new(f),
                        second: Box::new(s),
                    }),
                    (Some(f), None) => Some(f),
                    (None, Some(s)) => Some(s),
                    (None, None) => None,
                }
            }
        }
    }

    pub fn convert(self, target_id: &str, replacement: Self) -> Self {
        if self.pane_id().is_some_and(|id| id == target_id) {
            return replacement;
        }
        match self {
            Self::Split {
                id: sid,
                direction,
                ratio,
                first,
                second,
            } => Self::Split {
                id: sid,
                direction,
                ratio,
                first: Box::new(first.convert(target_id, replacement.clone())),
                second: Box::new(second.convert(target_id, replacement)),
            },
            other => other,
        }
    }

    pub fn set_ratio_by_split_id(&mut self, split_id: &str, new_ratio: f64) {
        match self {
            Self::Split {
                id,
                ratio,
                first,
                second,
                ..
            } => {
                if id == split_id {
                    *ratio = new_ratio.clamp(0.1, 0.9);
                } else {
                    first.set_ratio_by_split_id(split_id, new_ratio);
                    second.set_ratio_by_split_id(split_id, new_ratio);
                }
            }
            Self::Terminal { .. }
            | Self::Browser { .. }
            | Self::Editor { .. }
            | Self::Agent { .. }
            | Self::Canvas { .. } => {}
        }
    }

    pub fn compute_pane_rects(&self, rect: Rect) -> Vec<(Self, Rect)> {
        match self {
            Self::Terminal { .. }
            | Self::Browser { .. }
            | Self::Editor { .. }
            | Self::Agent { .. }
            | Self::Canvas { .. } => vec![(self.clone(), rect)],
            Self::Split {
                direction,
                ratio,
                first,
                second,
                ..
            } => {
                let (first_rect, second_rect) = split_rect(rect, *direction, *ratio);
                let mut result = first.compute_pane_rects(first_rect);
                result.extend(second.compute_pane_rects(second_rect));
                result
            }
        }
    }

    pub fn compute_dividers(&self, rect: Rect) -> Vec<DividerInfo> {
        match self {
            Self::Terminal { .. }
            | Self::Browser { .. }
            | Self::Editor { .. }
            | Self::Agent { .. }
            | Self::Canvas { .. } => vec![],
            Self::Split {
                id: sid,
                direction,
                ratio,
                first,
                second,
            } => {
                let (first_rect, second_rect) = split_rect(rect, *direction, *ratio);
                let _ = second_rect;
                let divider_rect = match direction {
                    SplitDirection::Horizontal => Rect {
                        left: first_rect.left + first_rect.width,
                        top: rect.top,
                        width: 0.0,
                        height: rect.height,
                    },
                    SplitDirection::Vertical => Rect {
                        left: rect.left,
                        top: first_rect.top + first_rect.height,
                        width: rect.width,
                        height: 0.0,
                    },
                };
                let mut result = vec![DividerInfo {
                    rect: divider_rect,
                    parent_rect: rect,
                    direction: *direction,
                    split_id: sid.clone(),
                }];
                result.extend(first.compute_dividers(first_rect));
                result.extend(second.compute_dividers(second_rect));
                result
            }
        }
    }

    pub fn all_pane_ids(&self) -> Vec<String> {
        match self {
            Self::Terminal { id, .. }
            | Self::Browser { id, .. }
            | Self::Editor { id, .. }
            | Self::Agent { id, .. }
            | Self::Canvas { id, .. } => vec![id.clone()],
            Self::Split { first, second, .. } => {
                let mut ids = first.all_pane_ids();
                ids.extend(second.all_pane_ids());
                ids
            }
        }
    }

    pub fn first_terminal_id(&self) -> Option<String> {
        match self {
            Self::Terminal { id, .. }
            | Self::Browser { id, .. }
            | Self::Editor { id, .. }
            | Self::Agent { id, .. }
            | Self::Canvas { id, .. } => Some(id.clone()),
            Self::Split { first, second, .. } => first
                .first_terminal_id()
                .or_else(|| second.first_terminal_id()),
        }
    }

    pub fn find_working_dir(&self, target_id: &str) -> Option<String> {
        match self {
            Self::Terminal {
                id, working_dir, ..
            } if id == target_id => Some(working_dir.clone()),
            Self::Editor { id, file_path, .. } if id == target_id => {
                std::path::Path::new(file_path)
                    .parent()
                    .map(|p| p.to_string_lossy().into_owned())
            }
            Self::Browser { id, .. } | Self::Agent { id, .. } | Self::Canvas { id, .. }
                if id == target_id =>
            {
                None
            }
            Self::Split { first, second, .. } => first
                .find_working_dir(target_id)
                .or_else(|| second.find_working_dir(target_id)),
            Self::Terminal { .. }
            | Self::Browser { .. }
            | Self::Editor { .. }
            | Self::Agent { .. }
            | Self::Canvas { .. } => None,
        }
    }
}

fn split_rect(rect: Rect, direction: SplitDirection, ratio: f64) -> (Rect, Rect) {
    match direction {
        SplitDirection::Horizontal => {
            let first_width = (rect.width - DIVIDER_SIZE_PCT) * ratio;
            let second_width = rect.width - DIVIDER_SIZE_PCT - first_width;
            (
                Rect {
                    left: rect.left,
                    top: rect.top,
                    width: first_width,
                    height: rect.height,
                },
                Rect {
                    left: rect.left + first_width + DIVIDER_SIZE_PCT,
                    top: rect.top,
                    width: second_width,
                    height: rect.height,
                },
            )
        }
        SplitDirection::Vertical => {
            let first_height = (rect.height - DIVIDER_SIZE_PCT) * ratio;
            let second_height = rect.height - DIVIDER_SIZE_PCT - first_height;
            (
                Rect {
                    left: rect.left,
                    top: rect.top,
                    width: rect.width,
                    height: first_height,
                },
                Rect {
                    left: rect.left,
                    top: rect.top + first_height + DIVIDER_SIZE_PCT,
                    width: rect.width,
                    height: second_height,
                },
            )
        }
    }
}
