use lx_dx::adapters::ansi;
use lx_dx::event::RuntimeEvent;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, EventCategory};

pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(frame.area());

    render_header(app, frame, chunks[0]);
    render_events(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
}

fn render_header(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let status_str = match &app.program_status {
        None => "starting",
        Some(Ok(_)) => "ok",
        Some(Err(_)) => "failed",
    };
    let status_color = match status_str {
        "ok" => Color::Green,
        "failed" => Color::Red,
        "starting" => Color::DarkGray,
        _ => Color::Yellow,
    };

    let content = Line::from(vec![
        Span::raw(&app.source_path),
        Span::raw(" | "),
        Span::styled(status_str, Style::default().fg(status_color)),
        Span::raw(format!(" | cost: ${:.4}", app.cumulative_cost)),
        Span::raw(format!(" | {}ms", app.elapsed_ms)),
    ]);

    let block = Block::default().borders(Borders::ALL).title("lx-tui");
    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_events(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let visible = app.visible_events();
    let total = app.events.len();
    let visible_count = visible.len();

    let items: Vec<ListItem<'_>> = visible
        .iter()
        .map(|event| {
            let spans = event_to_spans(event);
            ListItem::new(Line::from(spans))
        })
        .collect();

    let scroll_offset = if app.scroll > items.len().saturating_sub(1) {
        items.len().saturating_sub(1)
    } else {
        app.scroll
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Events ({visible_count}/{total})"));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);

    let inner_height = area.height.saturating_sub(2) as usize;
    if visible_count > inner_height && scroll_offset + inner_height < visible_count {
        let indicator = Paragraph::new("v").style(Style::default().fg(Color::DarkGray));
        let indicator_area = ratatui::layout::Rect {
            x: area.x + area.width - 2,
            y: area.y + area.height - 2,
            width: 1,
            height: 1,
        };
        frame.render_widget(indicator, indicator_area);
    }
}

fn render_footer(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let hotkeys = Line::from(vec![Span::raw(
        "a:AI  e:Emit  l:Log  s:Shell  m:Msg  g:Agent  p:Prog  r:Err  *:All  Tab:Agent  q:Quit",
    )]);

    let mut filter_spans = Vec::new();
    for (cat, label, color) in category_display_info() {
        let style = if app.filters.contains(&cat) {
            Style::default().fg(color)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        filter_spans.push(Span::styled(label, style));
        filter_spans.push(Span::raw("  "));
    }
    if let Some(ref agent) = app.agent_filter {
        filter_spans.push(Span::styled(
            format!("agent: {agent}"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    }
    let filter_line = Line::from(filter_spans);

    let paragraph = Paragraph::new(vec![hotkeys, filter_line]);
    frame.render_widget(paragraph, area);
}

fn category_display_info() -> Vec<(EventCategory, &'static str, Color)> {
    vec![
        (EventCategory::Ai, "AI", Color::Blue),
        (EventCategory::Emit, "Emit", Color::White),
        (EventCategory::Log, "Log", Color::Yellow),
        (EventCategory::Shell, "Shell", Color::DarkGray),
        (EventCategory::Messages, "Msg", Color::Cyan),
        (EventCategory::Agents, "Agent", Color::Green),
        (EventCategory::Progress, "Prog", Color::Magenta),
        (EventCategory::Errors, "Err", Color::Red),
    ]
}

fn event_to_spans(event: &RuntimeEvent) -> Vec<Span<'_>> {
    let formatted = ansi::format_event(event);
    let color = match event {
        RuntimeEvent::AiCallStart { .. } | RuntimeEvent::AiCallComplete { .. } => Color::Blue,
        RuntimeEvent::AiCallError { .. } | RuntimeEvent::Error { .. } => Color::Red,
        RuntimeEvent::ShellExec { .. } | RuntimeEvent::ShellResult { .. } => Color::DarkGray,
        RuntimeEvent::MessageSend { .. }
        | RuntimeEvent::MessageAsk { .. }
        | RuntimeEvent::MessageResponse { .. }
        | RuntimeEvent::UserPrompt { .. }
        | RuntimeEvent::UserResponse { .. } => Color::Cyan,
        RuntimeEvent::AgentSpawned { .. } | RuntimeEvent::AgentKilled { .. } => Color::Green,
        RuntimeEvent::Emit { .. } => Color::White,
        RuntimeEvent::Log { level, .. } => match level.as_str() {
            "warn" => Color::Yellow,
            "err" => Color::Red,
            "debug" => Color::DarkGray,
            _ => Color::Blue,
        },
        RuntimeEvent::Progress { .. }
        | RuntimeEvent::ProgramStarted { .. }
        | RuntimeEvent::ProgramFinished { .. }
        | RuntimeEvent::TraceSpanRecorded { .. } => Color::Magenta,
    };

    let clean = strip_ansi(&formatted);
    vec![Span::styled(clean, Style::default().fg(color))]
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}
