use crate::api::client::ForgejoClient;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::time::Duration;

pub struct NotificationPoller {
    poll_interval: Duration,
    last_check: HashMap<String, DateTime<Utc>>, // instance_id -> last check time
}

impl NotificationPoller {
    pub fn new(interval_minutes: u64) -> Self {
        Self {
            poll_interval: Duration::from_secs(interval_minutes * 60),
            last_check: HashMap::new(),
        }
    }

    // Poll a single instance for new notifications
    pub async fn poll_instance(
        &mut self,
        instance_id: &str,
        client: &ForgejoClient,
    ) -> Result<Vec<crate::api::models::Notification>, crate::api::error::ApiError> {
        let since = self.last_check.get(instance_id).copied();
        let notifications = client.list_notifications(since).await?;

        self.last_check.insert(instance_id.to_string(), Utc::now());
        Ok(notifications)
    }

    pub fn interval(&self) -> Duration {
        self.poll_interval
    }

    pub fn set_interval(&mut self, minutes: u64) {
        self.poll_interval = Duration::from_secs(minutes * 60);
    }
}
