use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, warn};

pub struct RemoteManager {
    recv: UnboundedReceiver<Result<Message, tokio_tungstenite::tungstenite::Error>>,
    send: UnboundedSender<Message>,

    current_upstream: Option<String>,
    nss_failed: bool,
}

impl RemoteManager {
    pub fn new(
        recv: UnboundedReceiver<Result<Message, tokio_tungstenite::tungstenite::Error>>,
        send: UnboundedSender<Message>,
    ) -> Self {
        Self {
            recv,
            send,
            current_upstream: None,
            nss_failed: false,
        }
    }

    pub async fn connect(&mut self) {
        self.send_connection_state_changed().await;

        loop {
            self.connect_loop().await;
        }
    }

    async fn connect_loop(&mut self) {
        if self.nss_failed {
            self.connect_ess().await.expect("connect_ess failed");
            return;
        }
        match self.connect_nss().await {
            Ok(_) => {
                self.nss_failed = false;
                warn!("nss connection closed")
            }
            Err(e) => {
                warn!("Failed to connect to NSS: {}", e);
                self.nss_failed = true;
                match self.connect_ess().await {
                    Ok(_) => {
                        warn!("ess connection closed")
                    }
                    Err(e) => {
                        error!("Failed to connect to ESS: {}", e);
                        self.current_upstream = None;
                    }
                }
            }
        }
    }

    async fn connect_nss(&mut self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        self.current_upstream = Some("nss".to_string());
        self.ws_connect(
            "wss://push.nanite-systems.net/streaming?environment=all&service-id=s:medkit2",
        )
        .await?;
        self.send_connection_state_changed().await;
        Ok(())
    }

    async fn connect_ess(&mut self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        self.current_upstream = Some("ess".to_string());
        self.ws_connect("wss://push.planetside2.com/streaming?environment=pc&service-id=s:medkit2")
            .await?;
        self.send_connection_state_changed().await;
        Ok(())
    }

    async fn ws_connect(&mut self, url: &str) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        todo!()
    }

    async fn send_connection_state_changed(&self) {
        self.send
            .unbounded_send(
                json!({
                    "connected": self.current_upstream.is_some(),
                    "service": "ess-demux",
                    "type": "essDemuxConnectionStateChanged",
                    "upstream": self.current_upstream,
                })
                .to_string()
                .into(),
            )
            .expect("send_connection_state_changed failed");
    }
}
