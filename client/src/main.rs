use tokio::io::{self, AsyncBufReadExt};
use tokio::task;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    println!("Connecting to ws://127.0.0.1:9000/ws ...");

    let (ws_stream, _) = connect_async("ws://127.0.0.1:9000/ws")
        .await
        .expect("Failed to connect");

    println!("Connected.");

    let (mut write, mut read) = ws_stream.split();

    // Task: incoming message handler
    let recv_task = task::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(m) => println!("<< {}", m),
                Err(e) => {
                    println!("WebSocket read error: {e}");
                    break;
                }
            }
        }
    });

    // Task: user input -> WebSocket
    let send_task = task::spawn(async move {
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        while let Ok(Some(line)) = stdin.next_line().await {
            if write.send(line.into()).await.is_err() {
                println!("WebSocket closed.");
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = recv_task => (),
        _ = send_task => (),
    }

    println!("Client exited.");
}
