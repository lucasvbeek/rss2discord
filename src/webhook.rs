use anyhow::Result;
use serde_json::json;

#[derive(Clone)]
pub struct Webhook {
    url: String,
}

impl Webhook {
    pub fn new(url: String) -> Self {
        Webhook {
            url
        }
    }

    pub async fn send_message(&self, message: &str) -> Result<()> {
        reqwest::Client::new().post(&self.url).json(&json!({
            "content": message
        })).send().await?;
        
        Ok(())
    }
}