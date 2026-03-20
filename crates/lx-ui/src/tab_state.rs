use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pane_tree::{PaneNode, SplitDirection};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerminalTab {
    pub id: String,
    pub name: String,
    pub root: PaneNode,
}

impl TerminalTab {
    pub fn new(name: &str, working_dir: &str) -> Self {
        let pane_id = Uuid::new_v4().to_string();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            root: PaneNode::Terminal {
                id: pane_id,
                working_dir: working_dir.to_string(),
                command: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
    Attention,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TabsState {
    pub tabs: Vec<TerminalTab>,
    pub active_tab_id: String,
    pub focused_pane_id: String,
    pub notifications: HashMap<String, NotificationLevel>,
}

impl TabsState {
    pub fn new(initial_tab: TerminalTab) -> Self {
        let active_id = initial_tab.id.clone();
        let focused = initial_tab.root.first_terminal_id().unwrap_or_default();
        Self {
            tabs: vec![initial_tab],
            active_tab_id: active_id,
            focused_pane_id: focused,
            notifications: HashMap::new(),
        }
    }

    pub fn add_tab(&mut self, tab: TerminalTab) {
        self.active_tab_id = tab.id.clone();
        if let Some(pid) = tab.root.first_terminal_id() {
            self.focused_pane_id = pid;
        }
        self.tabs.push(tab);
    }

    pub fn close_tab(&mut self, tab_id: &str) {
        self.tabs.retain(|t| t.id != tab_id);
        self.notifications.remove(tab_id);
        if self.active_tab_id == tab_id
            && let Some(first) = self.tabs.first()
        {
            self.active_tab_id = first.id.clone();
            if let Some(pid) = first.root.first_terminal_id() {
                self.focused_pane_id = pid;
            }
        }
    }

    pub fn split_active_pane(&mut self, direction: SplitDirection, new_pane: PaneNode) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == self.active_tab_id) {
            let target = self.focused_pane_id.clone();
            let root = std::mem::replace(
                &mut tab.root,
                PaneNode::Terminal {
                    id: String::new(),
                    working_dir: String::new(),
                    command: None,
                },
            );
            tab.root = root.split(&target, direction, new_pane);
        }
    }

    pub fn close_pane_in_active_tab(&mut self, pane_id: &str) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == self.active_tab_id) {
            let root = std::mem::replace(
                &mut tab.root,
                PaneNode::Terminal {
                    id: String::new(),
                    working_dir: String::new(),
                    command: None,
                },
            );
            if let Some(new_root) = root.close(pane_id) {
                tab.root = new_root;
                if self.focused_pane_id == pane_id
                    && let Some(pid) = tab.root.first_terminal_id()
                {
                    self.focused_pane_id = pid;
                }
            }
        }
    }

    pub fn set_active_tab_ratio(&mut self, split_id: &str, new_ratio: f64) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == self.active_tab_id) {
            let root = std::mem::replace(
                &mut tab.root,
                PaneNode::Terminal {
                    id: String::new(),
                    working_dir: String::new(),
                    command: None,
                },
            );
            tab.root = root.set_ratio_by_split_id(split_id, new_ratio);
        }
    }

    pub fn set_notification(&mut self, tab_id: &str, level: NotificationLevel) {
        self.notifications.insert(tab_id.to_string(), level);
    }

    pub fn get_notification(&self, tab_id: &str) -> Option<&NotificationLevel> {
        self.notifications.get(tab_id)
    }

    pub fn clear_tab_notifications(&mut self, tab_id: &str) {
        self.notifications.remove(tab_id);
    }
}
