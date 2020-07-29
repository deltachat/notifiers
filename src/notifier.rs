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
) -> Result<()> {
    let client = Client::certificate(&mut certificate, password, endpoint)?;

    let mut interval = async_std::stream::interval(std::time::Duration::from_secs(15));
    while let Some(_) = interval.next().await {
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

            let response = client.send(payload).await?;
            info!("sent: {} - {:?}", device_token, response);
        }
    }

    Ok(())
}
