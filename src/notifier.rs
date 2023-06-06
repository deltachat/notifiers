use a2::{
    Client, Endpoint, NotificationBuilder, NotificationOptions, Priority, SilentNotificationBuilder,
};
use anyhow::Result;
use async_std::prelude::*;
use log::*;

pub async fn start(
    db: &sled::Db,
    endpoint: Endpoint,
    mut certificate: std::fs::File,
    password: &str,
    topic: Option<&str>,
    interval: std::time::Duration,
) -> Result<()> {
    info!(
        "Waking up devices every {}",
        humantime::format_duration(interval)
    );
    let client = Client::certificate(&mut certificate, password, endpoint)?;

    // first wakeup on startup
    wakeup(db, &client, topic).await;

    // create interval
    let mut interval = async_std::stream::interval(interval);
    while interval.next().await.is_some() {
        wakeup(db, &client, topic).await;
    }

    Ok(())
}

async fn wakeup(db: &sled::Db, client: &Client, topic: Option<&str>) {
    let tokens = db
        .iter()
        .filter_map(|entry| match entry {
            Ok((key, _)) => Some(String::from_utf8(key.to_vec()).unwrap()),
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    info!("sending notifications to {} devices", tokens.len());

    for device_token in tokens {
        info!("notify: {}", device_token);

        let payload = SilentNotificationBuilder::new().build(
            &device_token,
            NotificationOptions {
                apns_priority: Some(Priority::Normal),
                apns_topic: topic,
                ..Default::default()
            },
        );

        match client.send(payload).await {
            Ok(res) => {
                match res.code {
                    200 => {
                        info!("delivered notification for {}", device_token);
                    }
                    410 => {
                        // no longer active
                        if let Err(err) = db.remove(&device_token) {
                            error!("failed to remove {}: {:?}", &device_token, err);
                        } else {
                            info!("removed inactive token: {}", &device_token);
                        }
                    }
                    _ => {
                        warn!("unexpected status: {:?}", res);
                    }
                }
            }
            Err(err) => {
                error!("failed to send notification: {}, {:?}", device_token, err);
            }
        }
    }
}
