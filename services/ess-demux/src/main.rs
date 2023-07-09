use futures::{pin_mut, select, FutureExt, StreamExt, TryStreamExt};
use futures_channel::mpsc::unbounded;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info};

use crate::remote_manager::RemoteManager;

mod remote_manager;

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (local_to_remote_tx, local_to_remote_rx) = unbounded();
    let (remote_to_local_tx, remote_to_local_rx) = unbounded();
    let (local_outgoing, local_incoming) = ws_stream.split();

    // Our client sent us a message, forward to ESS
    let local_to_remote = local_incoming.map(Ok).forward(local_to_remote_tx);

    // ESS sent us a message, forward to our client
    let remote_to_local = remote_to_local_rx.map(Ok).forward(local_outgoing);

    let upstream_connection = tokio::spawn(async move {
        let mut remote = RemoteManager::new(local_to_remote_rx, remote_to_local_tx.clone());
        remote.connect().await;
    })
    .fuse();

    pin_mut!(local_to_remote, remote_to_local, upstream_connection);
    select! {
        _ = local_to_remote => debug!("local_to_remote exited"),
        _ = remote_to_local => debug!("remote_to_local exited"),
        _ = upstream_connection => debug!("upstream_connection exited"),
    }

    info!("Client {} disconnected", addr);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let addr = format!(
        "0.0.0.0:{}",
        std::env::var("PORT").unwrap_or("8007".to_string())
    );

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
