use dioxus::prelude::*;

pub struct InboxBadge {
  pub count: Signal<usize>,
  pub has_unread: Memo<bool>,
}

pub fn use_inbox_badge() -> InboxBadge {
  let count = use_signal(|| 0usize);
  let has_unread = use_memo(move || count() > 0);
  InboxBadge { count, has_unread }
}
