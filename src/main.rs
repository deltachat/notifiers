use std::path::PathBuf;

use anyhow::Result;
use async_std::prelude::*;
use log::*;
use structopt::StructOpt;

use notifiers::server;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    sandbox: bool,
    #[structopt(long)]
    message: String,
    #[structopt(long, parse(from_os_str))]
    certificate_file: PathBuf,
    #[structopt(long)]
    password: String,
    #[structopt(long)]
    topic: Option<String>,
    #[structopt(long, default_value = "127.0.0.1")]
    server: String,
    #[structopt(long, default_value = "9000")]
    port: u16,
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
            async move { server::start(state2, opt.server.clone(), opt.port).await },
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
