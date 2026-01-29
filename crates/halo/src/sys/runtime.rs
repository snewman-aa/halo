use crate::events::AppEvent;
use async_channel::Sender;
use std::thread;
use tokio::runtime::Runtime;

pub fn start_background_services(tx: Sender<AppEvent>) {
    thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        rt.block_on(async {
            {
                let tx = tx.clone();
                tokio::spawn(async move {
                    crate::sys::server::run_server(tx).await;
                });
            }

            {
                let tx = tx.clone();
                tokio::spawn(async move {
                    crate::config::run_async_watcher(tx).await;
                });
            }

            std::future::pending::<()>().await;
        });
    });
}
