use std::sync::Arc;

use lx_dx::event::EventBus;
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::app::App;

pub fn start_program(source_path: &str, bus: Arc<EventBus>) {
    let path = source_path.to_string();
    std::thread::Builder::new()
        .name("lx-program".into())
        .spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to create program runtime");
            rt.block_on(async move {
                let langfuse = Arc::new(lx_dx::langfuse::LangfuseClient::from_env());
                let runner = lx_dx::runner::ProgramRunner::new(bus, langfuse);
                if let Err(e) = runner.run(&path).await {
                    eprintln!("program error: {e}");
                }
            });
        })
        .expect("failed to spawn program thread");
}

pub async fn run(app: &mut App, terminal: &mut Terminal<impl Backend>, bus: Arc<EventBus>) {
    let mut rx = bus.subscribe();

    let _ = terminal.draw(|f| crate::ui::render(app, f));

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        app.push_event(event);
                        let _ = terminal.draw(|f| crate::ui::render(app, f));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                }
            }
            _ = poll_crossterm() => {
                if let Ok(true) = crossterm::event::poll(std::time::Duration::ZERO)
                    && let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read()
                {
                    crate::input::handle_key(app, key);
                    let _ = terminal.draw(|f| crate::ui::render(app, f));
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
}

async fn poll_crossterm() {
    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
}
