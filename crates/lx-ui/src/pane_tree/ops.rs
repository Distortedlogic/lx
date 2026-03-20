use super::{DividerInfo, PaneNode, Rect, SplitDirection};

const DIVIDER_SIZE_PCT: f64 = 0.4;

impl PaneNode {
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
            Self::Terminal { id, .. } => Some(id.clone()),
            Self::Browser { .. }
            | Self::Editor { .. }
            | Self::Agent { .. }
            | Self::Canvas { .. } => None,
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
