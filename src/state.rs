use std::path::PathBuf;

use a2::{Client, Endpoint};
use anyhow::{Context as _, Result};
use async_std::sync::Arc;
use log::*;
use std::io::Seek;

#[derive(Debug, Clone)]
pub struct State {
    inner: Arc<InnerState>,
}

#[derive(Debug)]
pub struct InnerState {
    db: sled::Db,

    production_client: Client,

    sandbox_client: Client,
}

impl State {
    pub fn new(db: &PathBuf, mut certificate: std::fs::File, password: &str) -> Result<Self> {
        let db = sled::open(db)?;
        let production_client =
            Client::certificate(&mut certificate, password, Endpoint::Production)
                .context("Failed to create production client")?;
        certificate.rewind()?;
        let sandbox_client = Client::certificate(&mut certificate, password, Endpoint::Sandbox)
            .context("Failed to create sandbox client")?;

        info!("{} devices registered currently", db.len());

        Ok(State {
            inner: Arc::new(InnerState {
                db,
                production_client,
                sandbox_client,
            }),
        })
    }

    pub fn db(&self) -> &sled::Db {
        &self.inner.db
    }

    pub fn production_client(&self) -> &Client {
        &self.inner.production_client
    }

    pub fn sandbox_client(&self) -> &Client {
        &self.inner.sandbox_client
    }
}
