use futures::{SinkExt, StreamExt};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

use crate::pty_session;
use crate::ws_types::{ClientToTerminal, TerminalToClient};

pub async fn handle_terminal_ws<S>(
    ws_stream: WebSocketStream<S>,
    terminal_id: String,
    cols: u16,
    rows: u16,
    working_dir: Option<String>,
    command: Option<String>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut sink, mut stream) = ws_stream.split();

    let sess = match pty_session::get_or_create(
        &terminal_id,
        cols,
        rows,
        working_dir.as_deref(),
        command.as_deref(),
    ) {
        Ok(s) => s,
        Err(e) => {
            let msg = serde_json::to_string(&TerminalToClient::Error(e)).unwrap_or_default();
            let _ = sink.send(Message::Text(msg)).await;
            return;
        }
    };

    let (initial_output, mut output_rx) = sess.subscribe();

    let ready_msg =
        serde_json::to_string(&TerminalToClient::SessionReady { cols, rows }).unwrap_or_default();
    let _ = sink.send(Message::Text(ready_msg)).await;

    if !initial_output.is_empty() {
        let out_msg =
            serde_json::to_string(&TerminalToClient::Output(initial_output)).unwrap_or_default();
        let _ = sink.send(Message::Text(out_msg)).await;
    }

    loop {
        tokio::select! {
            result = output_rx.recv() => {
                match result {
                    Ok(data) => {
                        let msg = serde_json::to_string(&TerminalToClient::Output(data)).unwrap_or_default();
                        if sink.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                }
            }
            result = stream.next() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        let Ok(msg) = serde_json::from_str::<ClientToTerminal>(&text) else {
                            continue;
                        };
                        match msg {
                            ClientToTerminal::Input(data) => {
                                if sess.send_input(data).await.is_err() {
                                    break;
                                }
                            }
                            ClientToTerminal::Resize { cols, rows } => {
                                let _ = sess.resize(cols, rows);
                            }
                            ClientToTerminal::Close => {
                                pty_session::remove(&terminal_id);
                                let closed_msg = serde_json::to_string(&TerminalToClient::Closed).unwrap_or_default();
                                let _ = sink.send(Message::Text(closed_msg)).await;
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    pty_session::remove(&terminal_id);
}
