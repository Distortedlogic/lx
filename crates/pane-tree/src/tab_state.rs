use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::node::{Pane, PaneNode, SplitDirection};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tab<L: Clone> {
  pub id: String,
  pub title: String,
  root: Option<PaneNode<L>>,
}

impl<L: Pane> Tab<L> {
  pub fn new(title: &str, root_pane: L) -> Self {
    Self { id: Uuid::new_v4().to_string(), title: title.to_string(), root: Some(PaneNode::Leaf(root_pane)) }
  }

  pub fn with_root(title: &str, root: PaneNode<L>) -> Self {
    Self { id: Uuid::new_v4().to_string(), title: title.to_string(), root: Some(root) }
  }

  pub fn root(&self) -> &PaneNode<L> {
    self.root.as_ref().expect("Tab root must not be empty")
  }

  fn take_root(&mut self) -> PaneNode<L> {
    self.root.take().expect("Tab root must not be empty")
  }

  fn set_root(&mut self, root: PaneNode<L>) {
    self.root = Some(root);
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

impl NotificationLevel {
  fn ordinal(self) -> u8 {
    match self {
      Self::Info => 0,
      Self::Success => 1,
      Self::Warning => 2,
      Self::Attention => 3,
      Self::Error => 4,
    }
  }
}

impl PartialOrd for NotificationLevel {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for NotificationLevel {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.ordinal().cmp(&other.ordinal())
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaneNotification {
  pub level: NotificationLevel,
  pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TabsState<L: Clone> {
  pub tabs: Vec<Tab<L>>,
  pub active_tab_id: Option<String>,
  pub focused_pane_id: Option<String>,
  pub notifications: HashMap<String, PaneNotification>,
}

impl<L: Pane> TabsState<L> {
  pub fn new(initial_tab: Tab<L>) -> Self {
    let active_id = initial_tab.id.clone();
    let focused = initial_tab.root().first_leaf().map(|l| l.pane_id().to_owned());
    Self { tabs: vec![initial_tab], active_tab_id: Some(active_id), focused_pane_id: focused, notifications: HashMap::new() }
  }

  pub fn add_tab(&mut self, tab: Tab<L>) {
    self.active_tab_id = Some(tab.id.clone());
    self.focused_pane_id = tab.root().first_leaf().map(|l| l.pane_id().to_owned());
    self.tabs.push(tab);
  }

  pub fn close_tab(&mut self, tab_id: &str) {
    let was_active = self.active_tab_id.as_deref() == Some(tab_id);
    let idx = self.tabs.iter().position(|t| t.id == tab_id);
    if let Some(idx) = idx {
      self.tabs.remove(idx);
    }
    self.notifications.remove(tab_id);
    if was_active {
      let new_active = if let Some(i) = idx { if i < self.tabs.len() { Some(&self.tabs[i]) } else { self.tabs.last() } } else { self.tabs.first() };
      self.active_tab_id = new_active.map(|t| t.id.clone());
      self.focused_pane_id = new_active.and_then(|t| t.root().first_leaf().map(|l| l.pane_id().to_owned()));
    }
  }

  pub fn active_tab(&self) -> Option<&Tab<L>> {
    let id = self.active_tab_id.as_ref()?;
    self.tabs.iter().find(|t| &t.id == id)
  }

  pub fn set_active_and_focus(&mut self, tab_id: &str) {
    self.active_tab_id = Some(tab_id.to_owned());
    if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
      self.focused_pane_id = tab.root().first_leaf().map(|l| l.pane_id().to_owned());
    }
  }

  fn active_tab_mut(&mut self) -> Option<&mut Tab<L>> {
    let id = self.active_tab_id.clone()?;
    self.tabs.iter_mut().find(|t| t.id == id)
  }

  pub fn split_pane(&mut self, pane_id: &str, direction: SplitDirection, new_pane: PaneNode<L>) {
    if let Some(tab) = self.active_tab_mut() {
      let root = tab.take_root();
      tab.set_root(root.split(pane_id, direction, new_pane));
    }
  }

  pub fn close_pane_in_active_tab(&mut self, pane_id: &str) {
    let Some(active_id) = self.active_tab_id.clone() else {
      return;
    };
    let Some(tab) = self.active_tab_mut() else {
      return;
    };
    let root = tab.take_root();
    if let Some(new_root) = root.close(pane_id) {
      if self.focused_pane_id.as_deref() == Some(pane_id) {
        self.focused_pane_id = new_root.first_leaf().map(|l| l.pane_id().to_owned());
      }
      let tab = self.active_tab_mut().expect("active tab exists");
      tab.set_root(new_root);
    } else {
      self.close_tab(&active_id);
    }
  }

  pub fn convert_pane_in_active_tab(&mut self, pane_id: &str, new_node: PaneNode<L>) {
    if let Some(tab) = self.active_tab_mut() {
      let root = tab.take_root();
      tab.set_root(root.convert(pane_id, new_node));
    }
  }

  pub fn set_active_tab_ratio(&mut self, split_id: &str, new_ratio: f64) {
    if let Some(tab) = self.active_tab_mut() {
      let root = tab.take_root();
      tab.set_root(root.set_ratio_by_split_id(split_id, new_ratio));
    }
  }

  pub fn set_notification(&mut self, pane_id: &str, notification: PaneNotification) {
    self.notifications.insert(pane_id.to_string(), notification);
  }

  pub fn get_notification(&self, pane_id: &str) -> Option<&PaneNotification> {
    self.notifications.get(pane_id)
  }

  pub fn clear_tab_notifications(&mut self, tab_id: &str) {
    if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
      let pane_ids = tab.root().all_pane_ids();
      for pid in pane_ids {
        if let Some(n) = self.notifications.get(&pid)
          && matches!(n.level, NotificationLevel::Info | NotificationLevel::Success)
        {
          self.notifications.remove(&pid);
        }
      }
    }
  }
}
