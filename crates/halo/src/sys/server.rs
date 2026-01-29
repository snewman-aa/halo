use crate::events::AppEvent;
use async_channel::Sender;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/halo.sock";

pub async fn run_server(tx: Sender<AppEvent>) {
    // Cleanup old socket if it exists
    if std::fs::metadata(SOCKET_PATH).is_ok() {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind unix socket: {}", e);
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let reader = BufReader::new(&mut stream);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        match line.trim() {
                            "show" => {
                                let _ = tx.send(AppEvent::Show).await;
                            }
                            "hide" => {
                                let _ = tx.send(AppEvent::Hide).await;
                            }
                            _ => {}
                        }
                    }
                });
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
    }
}
