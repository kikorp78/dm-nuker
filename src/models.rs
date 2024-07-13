use clap::Parser;
use serde::Deserialize;
use serde_json::Value;

pub struct State {
    pub user_id: Option<String>,
    pub deleted_count: u32,
    pub args: Args,
    pub config: Config,
}

/// A tool for deleting messages from a Discord DM channel written in Rust.
#[derive(Parser)]
pub struct Args {
    /// Channel ID to delete messages from
    pub channel_id: String,

    /// Total number of messages to delete
    #[arg(short, long)]
    pub number: Option<u32>,

    /// Only delete messages with attachments
    #[arg(short = 'a', long, default_value_t = false)]
    pub only_attachments: bool,

    /// Delay between fetching each message group in milliseconds
    #[arg(long, default_value_t = 2000)]
    pub fetch_delay: u32,

    /// Delay between deleting each message in milliseconds
    #[arg(long, default_value_t = 3000)]
    pub delete_delay: u32,

    /// Filter to match messages against
    /// (ex. word | word,word2 | "sentence 1,sentence 2")
    #[arg(short, long, value_delimiter = ',')]
    pub filter: Vec<String>,

    /// ID before which to start deleting
    #[arg(long)]
    pub before_id: Option<String>,

    /// ID after which to stop deleting
    #[arg(long)]
    pub after_id: Option<String>,

    /// Output file to write deleted messages to
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub user_token: String,
}

#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub timestamp: String,
    pub author: MessageAuthor,
    pub attachments: Vec<Value>,
}

#[derive(Deserialize)]
pub struct MessageAuthor {
    pub id: String,
    pub username: String,
}
