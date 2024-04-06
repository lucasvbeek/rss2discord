mod config;
mod database;
mod feed;
mod receivers;
mod scheduler;

use std::env;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use log::info;

use crate::{config::Config, database::Database, scheduler::Scheduler};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Location of the config file
    #[arg(short, long, default_value_t = String::from("config.yaml"), env = "RSS2DISCORD_CONFIG")]
    config_location: String,
    #[arg(short, long, env = "RSS2DISCORD_DATABASE")]
    database: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::default().filter_or("RSS2DISCORD_LOG", "info");
    env_logger::init_from_env(env);

    dbg!(env::var("RSS2DISCORD_DATABASE")?);

    let args = Args::parse();

    dbg!(&args);

    let config = Config::load(args.config_location)?;

    info!(
        "Starting rss2discord v{} with {} feeds",
        VERSION,
        config.feeds.len()
    );


    let database = Database::init(&args.database).await?;

    let scheduler = Scheduler::init(config.feeds, database).await?;

    scheduler.start().await?;

    Ok(())
}
