use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, EventCategory};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('a') => app.toggle_filter(EventCategory::Ai),
        KeyCode::Char('e') => app.toggle_filter(EventCategory::Emit),
        KeyCode::Char('l') => app.toggle_filter(EventCategory::Log),
        KeyCode::Char('s') => app.toggle_filter(EventCategory::Shell),
        KeyCode::Char('m') => app.toggle_filter(EventCategory::Messages),
        KeyCode::Char('g') => app.toggle_filter(EventCategory::Agents),
        KeyCode::Char('p') => app.toggle_filter(EventCategory::Progress),
        KeyCode::Char('r') => app.toggle_filter(EventCategory::Errors),
        KeyCode::Char('*') => app.reset_filters(),
        KeyCode::Tab => app.cycle_agent(),
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Up => {
            app.scroll = app.scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = app.visible_events().len().saturating_sub(1);
            app.scroll = (app.scroll + 1).min(max);
        }
        KeyCode::PageUp => {
            app.scroll = app.scroll.saturating_sub(20);
        }
        KeyCode::PageDown => {
            let max = app.visible_events().len().saturating_sub(1);
            app.scroll = (app.scroll + 20).min(max);
        }
        KeyCode::Home => {
            app.scroll = 0;
        }
        KeyCode::End => {
            app.scroll = app.visible_events().len().saturating_sub(1);
        }
        _ => {}
    }
}
