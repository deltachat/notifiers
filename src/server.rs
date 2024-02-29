use anyhow::Result;
use log::*;
use serde::Deserialize;

use crate::state::State;

pub async fn start(state: State, server: String, port: u16) -> Result<()> {
    let mut app = tide::with_state(state);
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/register").post(register_device);

    info!("Listening on {server}:port");
    app.listen((server, port)).await?;
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct DeviceQuery {
    token: String,
}

async fn register_device(mut req: tide::Request<State>) -> tide::Result<tide::Response> {
    let query: DeviceQuery = req.body_json().await?;
    info!("register_device {}", query.token);

    let db = req.state().db();
    db.insert(query.token.as_bytes(), vec![1])?;
    db.flush_async().await?;

    Ok(tide::Response::new(tide::StatusCode::Ok))
}
