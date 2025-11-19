use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("Event-hub running on 127.0.0.1:7000");

    let clients: Arc<Mutex<HashMap<Uuid, Arc<TcpStream>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let id = Uuid::new_v4();

        println!("Client connected: {}", id);

        let clients_map = clients.clone();
        let stream = Arc::new(stream);

        clients_map.lock().unwrap().insert(id, stream.clone());

        tokio::spawn(async move {
            let _ = handle_client(id, stream, clients_map).await;
        });
    }
}

async fn handle_client(
    id: Uuid,
    stream: Arc<TcpStream>,
    clients: Arc<Mutex<HashMap<Uuid, Arc<TcpStream>>>>,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        let n = stream.read(&mut buf).await?;

        if n == 0 {
            clients.lock().unwrap().remove(&id);
            return Ok(());
        }

        for (other_id, other_stream) in clients.lock().unwrap().iter() {
            if *other_id != id {
                let _ = other_stream.write_all(&buf[..n]).await;
            }
        }
    }
}
