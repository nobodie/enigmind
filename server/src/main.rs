#![deny(clippy::all, clippy::unwrap_used)]

use std::{collections::HashMap, process::exit};

use axum::{
    extract::Query,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use enigmind_lib::setup::generate_game;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(hello))
        .route("/generate", get(generate))
        .route("/handshake", get(handshake));

    // run it with hyper on localhost:3000

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    exit(0);
}

async fn hello(Query(params): Query<HashMap<String, String>>) -> String {
    let mut s = "Parameters :\n".to_string();
    for p in params {
        s.push_str(format!("{} : {}\n", p.0, p.1).as_str());
    }
    s
}

async fn handshake() -> Response {
    Json("ok").into_response()
}

fn extract_u8_param_or(params: &HashMap<String, String>, name: &str, default: u8) -> u8 {
    params
        .get(&name.to_string())
        .unwrap_or(&String::new())
        .parse::<u8>()
        .unwrap_or(default)
}

async fn generate(Query(params): Query<HashMap<String, String>>) -> Response {
    let base = extract_u8_param_or(&params, "base", 5);
    let column_count = extract_u8_param_or(&params, "column_count", 3);

    match generate_game(base, column_count) {
        Ok(game) => Json(game).into_response(),
        Err(e) => Json(e.to_string()).into_response(),
    }
}
