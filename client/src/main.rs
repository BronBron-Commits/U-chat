use reqwest::Client;
use serde::{Serialize, Deserialize};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

#[derive(Serialize)]
struct LoginReq {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResp {
    token: String,
}

#[tokio::main]
async fn main() {
    let http = Client::new();

    println!("Logging in to Auth API...");
    let resp = http.post("http://127.0.0.1:9200/login")
        .json(&LoginReq {
            username: "user".into(),
            password: "password".into(),
        })
        .send()
        .await
        .unwrap()
        .json::<LoginResp>()
        .await
        .unwrap();

    println!("TOKEN = {}", resp.token);

    let ws_url = format!("ws://127.0.0.1:9000/ws?token={}", resp.token);
    println!("Connecting to {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            println!("WS RECEIVED: {:?}", msg);
        }
    });

    write.send(Message::Text("{\"msg\":\"hello world\"}".into()))
        .await
        .unwrap();

    println!("Client finished sending message.");
    loop { tokio::time::sleep(std::time::Duration::from_secs(1)).await; }
}
