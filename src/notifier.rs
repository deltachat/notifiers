use a2::{
    Client, DefaultNotificationBuilder, Endpoint, Error::ResponseError, NotificationBuilder,
    NotificationOptions, Priority,
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

        // Sent silent notification.
        // According to <https://developer.apple.com/documentation/usernotifications/generating-a-remote-notification>
        // to send a silent notification you need to set background notification flag `content-available` to 1
        // and don't include `alert`, `badge` or `sound`.
        let payload = DefaultNotificationBuilder::new()
            .set_content_available()
            .build(
                &device_token,
                NotificationOptions {
                    // Normal priority (5) means
                    // "send the notification based on power considerations on the user’s device".
                    // <https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns>
                    apns_priority: Some(Priority::Normal),
                    apns_topic: topic,
                    ..Default::default()
                },
            );

        match client.send(payload).await {
            Ok(res) => match res.code {
                200 => {
                    info!("delivered notification for {}", device_token);
                }
                _ => {
                    warn!("unexpected status: {:?}", res);
                }
            },
            Err(ResponseError(res)) => {
                info!("Removing token {} due to error {:?}.", &device_token, res);
                if let Err(err) = db.remove(&device_token) {
                    error!("failed to remove {}: {:?}", &device_token, err);
                }
            }
            Err(err) => {
                error!("failed to send notification: {}, {:?}", device_token, err);
            }
        }
    }
}
