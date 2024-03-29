use std::path::PathBuf;

use anyhow::{Context, Result};
use async_std::prelude::*;
use structopt::StructOpt;

use notifiers::{notifier, server, state};

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to the certificate file PKS12.
    #[structopt(long, parse(from_os_str))]
    certificate_file: PathBuf,
    /// Password for the certificate file.
    #[structopt(long)]
    password: String,
    /// The topic for the notification.
    #[structopt(long)]
    topic: Option<String>,
    /// The host on which to start the server.
    #[structopt(long, default_value = "127.0.0.1")]
    host: String,
    /// The port on which to start the server.
    #[structopt(long, default_value = "9000")]
    port: u16,
    /// The path to the database file.
    #[structopt(long, default_value = "notifiers.db", parse(from_os_str))]
    db: PathBuf,
    #[structopt(long, default_value = "20m", parse(try_from_str = humantime::parse_duration))]
    interval: std::time::Duration,
}

#[async_std::main]
async fn main() -> Result<()> {
    femme::start();

    let opt = Opt::from_args();
    let certificate = std::fs::File::open(&opt.certificate_file).context("invalid certificate")?;

    let state = state::State::new(&opt.db, certificate, &opt.password, opt.topic.clone())?;

    let state2 = state.clone();
    let host = opt.host.clone();
    let port = opt.port;
    let server = async_std::task::spawn(async move { server::start(state2, host, port).await });

    let notif = async_std::task::spawn(async move { notifier::start(state, opt.interval).await });

    server.try_join(notif).await?;

    Ok(())
}
