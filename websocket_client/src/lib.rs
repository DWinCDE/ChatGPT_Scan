use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

#[derive(Clone)]
pub struct WebSocketClient {
    url: String,
    reconnect_delay: Duration,
    initial_message: Option<String>,
}

impl WebSocketClient {
    pub fn new(url: &str, initial_message: Option<String>) -> Self {
        Self {
            url: url.to_string(),
            reconnect_delay: Duration::from_secs(5),
            initial_message,
        }
    }

    pub async fn start<F>(&self, mut callback: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        loop {
            match self.connect_and_listen(&mut callback).await {
                Ok(_) => {
                    eprintln!("Disconnected. Attempting to reconnect...");
                }
                Err(e) => {
                    eprintln!(
                        "Connection error: {}. Reconnecting in {:?}",
                        e, self.reconnect_delay
                    );
                    time::sleep(self.reconnect_delay).await;
                }
            }
        }
    }

    async fn connect_and_listen<F>(&self, callback: &mut F) -> Result<(), TungsteniteError>
    where
        F: FnMut(String) + Send + 'static,
    {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();
        let (ping_tx, mut ping_rx) = mpsc::channel::<()>(1);

        if let Some(initial_message) = &self.initial_message {
            write.send(Message::Text(initial_message.clone())).await?;
        }

        tokio::spawn(async move {
            loop {
                time::sleep(Duration::from_secs(30)).await;
                if ping_tx.send(()).await.is_err() {
                    break;
                }
            }
        });

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            callback(text);
                        }
                        Some(Ok(Message::Ping(_))) => {
                            write.send(Message::Pong(vec![])).await?;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => return Err(e),
                        None => return Ok(()),
                    }
                }
                _ = ping_rx.recv() => {
                    write.send(Message::Ping(vec![])).await?;
                }
            }
        }
    }
}
