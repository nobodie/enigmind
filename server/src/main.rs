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
        .route("/generate", get(generate))
        .route("/ping", get(ping));

    // run it with hyper on localhost:3000

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    exit(0);
}

async fn ping() -> Response {
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
    let difficulty_pct = extract_u8_param_or(&params, "difficulty_pct", 10);

    match generate_game(base, column_count, difficulty_pct) {
        Ok(game) => Json(game).into_response(),
        Err(e) => Json(e.to_string()).into_response(),
    }
}
