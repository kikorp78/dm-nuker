use std::fs;

use clap::Parser;
use functions::{fetch_current_user_id, start};
use models::{Args, Config, State};
use utils::log_error;

mod functions;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    let args = Args::parse();

    let config_file = fs::read_to_string("config.toml").unwrap_or_else(|e| {
        log_error(e, "Failed reading config.toml file. Does it exist?", true);
        unreachable!();
    });

    let config = toml::from_str::<Config>(&config_file).unwrap_or_else(|e| {
        log_error(e, "Failed parsing config.toml file. Please ensure it contains channel_id and user_token fields.", true);
        unreachable!();
    });

    let mut state = State {
        user_id: None,
        deleted_count: 0,
        args,
        config,
    };

    let client = reqwest::Client::new();

    let current_user_id = fetch_current_user_id(&client, &state).await;

    state.user_id = Some(current_user_id);

    start(&client, &mut state).await;
}
