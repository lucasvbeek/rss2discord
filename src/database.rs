use std::{collections::BTreeMap, str::FromStr, time::{Duration, Instant}};

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use log::info;
use serde_json::json;
use sqlx::{postgres::{PgConnectOptions, PgPoolOptions}, Executor, Pool, Postgres, QueryBuilder, Row};

use crate::config::ConfigDatabase;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>
}

#[derive(Clone)]
pub struct DatabaseFeedItem {
    pub feed_name: String,
    pub external_id: String,
    pub published_at: DateTime<FixedOffset>,
    pub variables: BTreeMap<String, String>
}

impl Database {
    pub async fn init(config: ConfigDatabase)-> Result<Self> {
        let options = PgConnectOptions::from_str(&config.connection)?;
        let pool = PgPoolOptions::new().max_connections(50).connect_with(options).await?;
        let conn = pool.acquire().await?;
        let db = Database { pool };

        info!("Connected to postgres version {}", conn.server_version_num().unwrap_or(0));

        info!("Running database migrations...");
        let elapsed = db.migrate().await?;
        info!("Database migrations completed in {:?}", elapsed);

        Ok(db)
    }

    async fn migrate(&self) -> Result<Duration> {
        let migrations_start = Instant::now();
        sqlx::migrate!().run(&self.pool).await?;
        Ok(migrations_start.elapsed())
    }

    pub async fn insert_and_select_feed_items(&self, items: &Vec<DatabaseFeedItem>) -> Result<Vec<String>> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO feed_items (feed_name, external_id, published_at, variables) ");

        query_builder.push_values(items, |mut b, new_item| {
            b.push_bind(new_item.feed_name.clone())
             .push_bind(new_item.external_id.clone())
             .push_bind(new_item.published_at)
             .push_bind(json!(new_item.variables));
        });

        query_builder.push("ON CONFLICT (feed_name, external_id) DO NOTHING RETURNING external_id");

        let query = query_builder.build();

        let rows = self.pool.fetch_all(query).await?;

        Ok(rows.iter().map(|r| r.get(0)).collect())
    }
}