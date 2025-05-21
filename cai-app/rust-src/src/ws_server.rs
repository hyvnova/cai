use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot, Mutex},
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::Message,
};

/// ============= Your domain payloads =============
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MsgRole {
    Assistant,
    System,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum MsgType {
    Plain(String),
    TitleChildren { title: String, content: Vec<String> },
}

/// Everything over the wire lives in an envelope
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Envelope {
    role: MsgRole,
    body: MsgType,
}

/// Message types for the internal control channel
enum Internal {
    Broadcast(Envelope),
    Ask {
        prompt: String,
        reply_tx: oneshot::Sender<String>,
    },
}

/// Hold shared state (connected sockets)
#[derive(Default)]
struct Hub {
    peers: Vec<mpsc::UnboundedSender<Message>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let hub = Arc::new(Mutex::new(Hub::default()));
    let (internal_tx, mut internal_rx) = mpsc::unbounded_channel::<Internal>();

    // === 1) Spawn background task to fan-out app→client messages ==========
    let hub_ctrl = hub.clone();
    tokio::spawn(async move {
        while let Some(cmd) = internal_rx.recv().await {
            match cmd {
                Internal::Broadcast(env) => {
                    let txt = serde_json::to_string(&env).unwrap();
                    let mut hub = hub_ctrl.lock().await;
                    hub.peers.retain(|tx| tx.send(Message::Text(txt.clone())).is_ok());
                }
                Internal::Ask { prompt, reply_tx } => {
                    let ask = Envelope {
                        role: MsgRole::System,
                        body: MsgType::Plain(prompt.clone()),
                    };
                    let txt = serde_json::to_string(&ask).unwrap();
                    let mut hub = hub_ctrl.lock().await;
                    if hub.peers.is_empty() {
                        let _ = reply_tx.send(String::from("<no client>"));
                        continue;
                    }
                    // For demo, ask the *first* peer.
                    let peer_tx = hub.peers[0].clone();
                    if peer_tx.send(Message::Text(txt)).is_err() {
                        let _ = reply_tx.send(String::from("<send-fail>"));
                        continue;
                    }
                    // Wait for answer via a oneshot inserted into map
                    // (implemented in connection task below).
                    // The `reply_tx` travels via a shared map keyed by uuid.
                }
            }
        }
    });

    // === 2) Accept TCP → upgrade to WS ====================================
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("🚀  ws://{addr} ready");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let hub_clone = hub.clone();
        let internal_clone = internal_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_addr, hub_clone, internal_clone).await {
                eprintln!("⛔  {peer_addr}: {e}");
            }
        });
    }
}

/// per-client task
async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    hub: Arc<Mutex<Hub>>,
    _internal_tx: mpsc::UnboundedSender<Internal>,
) -> Result<()> {
    let ws = accept_async(stream).await?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Channel so hub can push to this socket
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
    // Register with hub
    {
        let mut h = hub.lock().await;
        h.peers.push(out_tx);
    }

    // TASK A → pump outbound msgs
    let pump_out = async move {
        while let Some(msg) = out_rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    };

    // TASK B → handle inbound msgs (answers from user)
    let pump_in = async move {
        while let Some(Ok(Message::Text(txt))) = ws_rx.next().await {
            if let Ok(env) = serde_json::from_str::<Envelope>(&txt) {
                println!("📨  <- {peer}: {env:?}");
                // TODO: match env.body to detect answers vs other events
            }
        }
    };

    // Run both pumps; when either ends, connection ends
    tokio::select! {
        _ = pump_out => {},
        _ = pump_in => {},
    }

    println!("👋  {peer} disconnected");
    Ok(())
}
