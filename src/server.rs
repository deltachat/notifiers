use std::path::PathBuf;

use anyhow::Result;
use async_std::sync::Arc;
use log::*;
use serde::Deserialize;

pub async fn start(state: State, server: String, port: u16) -> Result<()> {
    let mut app = tide::with_state(state);
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/register").post(register_device);

    let addr = format!("{}:{}", server, port);
    println!("Listening on {}", &addr);
    app.listen(addr).await?;
    Ok(())
}

#[derive(Debug)]
pub struct State {
    inner: Arc<InnerState>,
}

impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug)]
pub struct InnerState {
    db: sled::Db,
}

impl State {
    pub fn new(db: &PathBuf) -> Result<Self> {
        let db = sled::open(db)?;
        info!("{} devices registered currently", db.len());

        Ok(State {
            inner: Arc::new(InnerState { db }),
        })
    }

    pub fn db(&self) -> &sled::Db {
        &self.inner.db
    }
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
