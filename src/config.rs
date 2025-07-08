use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub feeds: Vec<ConfigFeed>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeed {
    pub id: String,
    pub rss_url: String,
    pub interval: u64,
    pub guid_regex: Option<String>,
    pub receivers: Vec<ConfigFeedReceiver>,
    pub user_agent: Option<String>,
    pub atom: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeedReceiver {
    #[serde(rename = "type")]
    pub receiver_type: ConfigFeedReceiverType,
    pub discord: ConfigFeedDiscordReceiver,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFeedReceiverType {
    Discord,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeedDiscordReceiver {
    pub webhook_url: String,
    pub content: Option<String>,
    #[serde(default)]
    pub embeds: Vec<ConfigFeedDiscordReceiverEmbed>,
    #[serde(default)]
    pub overrides: Vec<ConfigFeedDiscordReceiverOverride>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeedDiscordReceiverOverride {
    pub regex: String,
    pub field: String,
    pub webhook_url: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeedDiscordReceiverEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<ConfigFeedDiscordReceiverEmbedField>,
    pub footer: Option<String>,
    pub image: Option<String>,
    pub thumbnail: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFeedDiscordReceiverEmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

impl Config {
    pub fn load(path: String) -> Result<Self> {
        let config: Self = confy::load_path(PathBuf::from(path).as_path())?;
        Ok(config)
    }
}
