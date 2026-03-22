use crate::node::{DividerInfo, Pane, PaneNode, Rect, SplitDirection};

const DIVIDER_SIZE_PCT: f64 = 0.4;

impl<L: Pane> PaneNode<L> {
    pub fn compute_pane_rects(&self, rect: Rect) -> Vec<(&L, Rect)> {
        match self {
            Self::Leaf(leaf) => vec![(leaf, rect)],
            Self::Split { direction, ratio, first, second, .. } => {
                let (first_rect, second_rect) = split_rect(rect, *direction, *ratio);
                let mut result = first.compute_pane_rects(first_rect);
                result.extend(second.compute_pane_rects(second_rect));
                result
            }
        }
    }

    pub fn compute_dividers(&self, rect: Rect) -> Vec<DividerInfo> {
        match self {
            Self::Leaf(_) => vec![],
            Self::Split { id, direction, ratio, first, second } => {
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
                    split_id: id.clone(),
                }];
                result.extend(first.compute_dividers(first_rect));
                result.extend(second.compute_dividers(second_rect));
                result
            }
        }
    }
}

fn split_rect(rect: Rect, direction: SplitDirection, ratio: f64) -> (Rect, Rect) {
    match direction {
        SplitDirection::Horizontal => {
            let first_width = (rect.width - DIVIDER_SIZE_PCT) * ratio;
            let second_width = rect.width - DIVIDER_SIZE_PCT - first_width;
            (
                Rect { left: rect.left, top: rect.top, width: first_width, height: rect.height },
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
                Rect { left: rect.left, top: rect.top, width: rect.width, height: first_height },
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
