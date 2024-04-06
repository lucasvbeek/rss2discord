use anyhow::Result;

use crate::database::DatabaseFeedItem;

pub mod discord;
pub trait Receivable {
    async fn send_item(&self, item: &DatabaseFeedItem) -> Result<()>;
}
