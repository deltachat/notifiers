use std::path::PathBuf;

use anyhow::Result;
use async_std::prelude::*;
use log::*;
use structopt::StructOpt;

use notifiers::server;

#[derive(Debug, StructOpt)]
struct Opt {
    /// If set, this will use the sandbox servers, instead of the production ones.
    #[structopt(short, long)]
    sandbox: bool,
    /// The message for the notification to be sent.
    #[structopt(long)]
    message: String,
    /// Path to the certificate file.
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
}

#[async_std::main]
async fn main() -> Result<()> {
    femme::start();

    let opt = Opt::from_args();

    let state = server::State::new(&opt.db)?;

    let state2 = state.clone();
    let server =
        async_std::task::spawn(
            async move { server::start(state2, opt.host.clone(), opt.port).await },
        );

    async_std::task::spawn(async move {
        let db = state.db();
        let mut interval = async_std::stream::interval(std::time::Duration::from_secs(15));
        while let Some(_) = interval.next().await {
            info!("sending notifications");
            for res in db.iter() {
                if let Ok((key, _)) = res {
                    let key = String::from_utf8(key.to_vec()).unwrap();
                    info!("notify {}", key);
                }
            }
        }
    });

    server.await?;

    Ok(())
}
