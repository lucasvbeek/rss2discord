use std::time::Duration;

use anyhow::Result;
use log::warn;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{config::ConfigFeed, database::Database, feed::Feed};

pub struct Scheduler {
    scheduler: JobScheduler
}

impl Scheduler {
    pub async fn init(feeds: Vec<ConfigFeed>, database: Database) -> Result<Self> {
        let scheduler = JobScheduler::new().await.unwrap();
    
        for feed_config in feeds.clone() {
            let feed = Feed::from_config(feed_config.clone());
            let database = database.clone();

            let job = Job::new_repeated_async(Duration::from_secs(feed_config.interval), move |_, _| {
                let feed = feed.clone();
                let database = database.clone();
                Box::pin(async move { 
                    if let Err(e) = feed.process(&database).await { warn!("Error processing feed {}: {}", feed.id, e) }
                })
            })?;

            scheduler.add(job).await?;
        }
    
        Ok(Scheduler { scheduler })
    }

    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await?;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}