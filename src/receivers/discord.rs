use crate::config::ConfigFeedDiscordReceiver;
use serde_json::{json, Value};

use super::Receivable;

pub struct DiscordReceiver {
    pub config: ConfigFeedDiscordReceiver,
}

pub type JsonObject = serde_json::Map<String, serde_json::Value>;

impl Receivable for DiscordReceiver {
    async fn send_item(&self, item: &crate::database::DatabaseFeedItem) -> anyhow::Result<()> {
        let mut message = JsonObject::new();

        if let Some(content) = &self.config.content {
            message.insert(String::from("content"), Value::String(item.sub(content)));
        }

        let embeds: Vec<JsonObject> = self
            .config
            .embeds
            .iter()
            .map(|e| {
                let mut embed = JsonObject::new();

                if let Some(title) = e.title.clone() {
                    embed.insert(
                        String::from("title"),
                        Value::String(trunc(&item.sub(&title), 256)),
                    );
                }

                if let Some(description) = e.description.clone() {
                    embed.insert(
                        String::from("description"),
                        Value::String(trunc(&item.sub(&description), 4096)),
                    );
                }

                if let Some(url) = e.url.clone() {
                    embed.insert(String::from("url"), Value::String(item.sub(&url)));
                }

                if let Some(image) = e.image.clone() {
                    embed.insert(String::from("image"), json!({"url": &item.sub(&image)}));
                }

                if let Some(thumbnail) = e.thumbnail.clone() {
                    embed.insert(
                        String::from("thumbnail"),
                        json!({"url": &item.sub(&thumbnail)}),
                    );
                }

                if let Some(footer) = e.footer.clone() {
                    embed.insert(
                        String::from("footer"),
                        json!({"text": trunc(&item.sub(&footer), 2048)}),
                    );
                }

                let fields: Vec<Value> = e
                    .fields
                    .iter()
                    .map(|f| {
                        json!({
                            "name": trunc(&item.sub(&f.name), 256),
                            "value": trunc(&item.sub(&f.value), 1024),
                            "inline": f.inline
                        })
                    })
                    .collect();

                embed.insert(String::from("fields"), fields.into());

                embed
            })
            .collect();

        message.insert(String::from("embeds"), embeds.into());

        dbg!(&message);

        reqwest::Client::new()
            .post(&self.config.webhook_url)
            .json(&message)
            .send()
            .await?;
        Ok(())
    }
}

impl DiscordReceiver {
    pub fn new(config: &ConfigFeedDiscordReceiver) -> Self {
        DiscordReceiver {
            config: config.clone(),
        }
    }
}

fn trunc(input: &str, len: usize) -> String {
    let mut str = input.to_owned();
    str.truncate(len);
    str
}
