use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use serde::Deserialize;

use uchat_proto::jwt::create_token;
use uchat_proto::events::ServerEvent;

use serde_json;
use anyhow::Result;

#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:9200".parse().unwrap();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle_request))
    });

    println!("auth-api running on http://{}", addr);

    Server::bind(&addr)
        .serve(make_svc)
        .await?;

    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/login") => handle_login(req).await,
        _ => Ok(not_found()),
    }
}

async fn handle_login(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let whole_body = hyper::body::to_bytes(req.into_body()).await?;
    let login: LoginReq = match serde_json::from_slice(&whole_body) {
        Ok(v) => v,
        Err(_) => return Ok(json_error("invalid json")),
    };

    // TODO: password verification — currently accept anything
    let token = create_token("MY_SECRET_KEY", &login.username);

    let response = ServerEvent::LoginOk { token };
    let json = serde_json::to_string(&response).unwrap();

    Ok(json_ok(json))
}

fn json_ok(body: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

fn json_error(msg: &str) -> Response<Body> {
    let err = ServerEvent::Error { details: msg.into() };
    let json = serde_json::to_string(&err).unwrap();

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .unwrap()
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("not found"))
        .unwrap()
}
