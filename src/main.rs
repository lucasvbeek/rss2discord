mod config;
mod database;
mod scheduler;
mod feed;
mod webhook;

use clap::Parser;
use env_logger::Env;
use log::info;
use anyhow::Result;

use crate::{config::Config, database::Database, scheduler::Scheduler};

const VERSION: &str = env!("CARGO_PKG_VERSION");


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Location of the config file
    #[arg(short, long, default_value_t = String::from("config.toml"))]
    config_location: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::default()
    .filter_or("RSS2DISCORD_LOG", "info");
    env_logger::init_from_env(env);

    let args = Args::parse();
    let config = Config::load(args.config_location)?;

    info!("Starting rss2discord v{}", VERSION);

    let database = Database::init(config.database).await?;

    let scheduler = Scheduler::init(config.feeds, database).await?;
    
    scheduler.start().await?;

    Ok(())
}
