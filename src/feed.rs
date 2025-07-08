use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use chrono_tz::Tz;
use fancy_regex::Regex;
use log::debug;
use reqwest::Method;
use rss::{extension::Extension, Channel, Guid, Item};

use crate::{
    config::{ConfigFeed, ConfigFeedReceiver, ConfigFeedReceiverType},
    database::{Database, DatabaseFeedItem},
    receivers::{discord::DiscordReceiver, Receivable},
};

#[derive(Clone)]
pub struct Feed {
    pub id: String,
    url: String,
    user_agent: Option<String>,
    receivers: Vec<ConfigFeedReceiver>,
    regex: Option<Regex>,
}

impl Feed {
    pub fn from_config(config: ConfigFeed) -> Self {
        Feed {
            id: config.id,
            url: config.rss_url,
            receivers: config.receivers,
            user_agent: config.user_agent,
            regex: config
                .guid_regex
                .map(|re| Regex::new(&re).expect("Invalid regex")),
        }
    }

    pub async fn process(&self, database: &Database) -> Result<()> {
        let channel = fetch_and_parse_feed(&self.url, &self.user_agent).await?;

        debug!("Fetching feed {} {}", self.id, self.url);

        let mut items: Vec<DatabaseFeedItem> = channel
            .items
            .iter()
            .filter(|i| i.link.is_some())
            .map(|i| (i.clone(), parse_variables_from_item(i)))
            .map(|i| DatabaseFeedItem {
                feed_name: self.id.clone(),
                external_id: get_unique_id_from_item(&i.0, &self.regex),
                published_at: parse_datetime_from_item(&i.0),
                variables: i.1,
            })
            .collect();

        items.sort_by_key(|i| i.published_at);

        debug!("Received {} items from feed {}", items.len(), self.id);

        let new_item_ids = database.insert_and_select_feed_items(&items).await?;

        for item in items
            .into_iter()
            .filter(|i| new_item_ids.contains(&i.external_id))
        {
            debug!("Sending notification for new item {}", item.external_id);

            for receiver in &self.receivers {
                match receiver.receiver_type {
                    ConfigFeedReceiverType::Discord => {
                        DiscordReceiver::new(&receiver.discord)
                            .send_item(&item)
                            .await?
                    }
                }
            }
        }

        Ok(())
    }
}

async fn fetch_and_parse_feed(feed: &str, user_agent: &Option<String>) -> Result<Channel> {
    let client = reqwest::Client::new();

    let req = match user_agent {
        Some(agent) => client
            .request(Method::GET, feed)
            .header(reqwest::header::USER_AGENT, agent),
        None => client.request(Method::GET, feed),
    };

    let resp = req.send().await?;

    if resp.status().as_u16() != 200 {
        return Err(anyhow!("unexpected statuscode {}", resp.status()));
    }
    let content = resp.bytes().await?;

    let channel = Channel::read_from(&content[..])?;

    Ok(channel)
}

fn get_unique_id_from_item(item: &Item, regex: &Option<Regex>) -> String {
    let mut guid = item
        .guid
        .clone()
        .unwrap_or(Guid {
            value: item.link.clone().expect("Item has no link"),
            permalink: false,
        })
        .value;

    if let Some(regex) = regex {
        if let Ok(Some(m)) = regex.find(&guid) {
            guid = m.as_str().to_owned();
        }
    }

    guid
}

fn parse_datetime_from_item(item: &Item) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc2822(&item.pub_date.clone().unwrap_or_default()).unwrap_or_default()
}

fn parse_variables_from_item(item: &Item) -> BTreeMap<String, String> {
    let mut variables: Vec<(String, String)> = Vec::new();

    if let Some(title) = &item.title {
        variables.push((String::from("title"), title.clone()));
    }

    if let Some(description) = &item.description {
        variables.push((String::from("description"), description.clone()));
    }

    if let Some(link) = &item.link {
        variables.push((String::from("link"), link.clone()));
    }

    if let Some(comments) = &item.comments {
        variables.push((String::from("comments"), comments.clone()));
    }

    if let Some(pub_date) = &item.pub_date {
        let datetime = DateTime::parse_from_rfc2822(pub_date).unwrap_or_default();
        let datetime = datetime.with_timezone(&Tz::UTC);
        variables.push((
            String::from("pub_date"),
            datetime.format("%v %R %Z").to_string(),
        ));
    }

    variables.push((
        String::from("categories"),
        item.categories
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>()
            .join(", "),
    ));

    let extentions: Vec<&Extension> = item
        .extensions
        .values()
        .flatten()
        .flat_map(|(_, m)| m)
        .collect();

    variables.append(
        &mut extentions
            .iter()
            .flat_map(|ext| {
                ext.attrs
                    .iter()
                    .map(|(key, value)| {
                        (
                            format!("{}_{}", ext.name.replace(':', "_"), key),
                            value.clone(),
                        )
                    })
                    .collect::<Vec<(String, String)>>()
            })
            .collect(),
    );

    variables.append(
        &mut extentions
            .iter()
            .filter(|ext| ext.value.is_some())
            .map(|ext| {
                (
                    ext.name.clone().replace(':', "_"),
                    ext.value.clone().unwrap(),
                )
            })
            .collect(),
    );

    variables.into_iter().collect()
}
