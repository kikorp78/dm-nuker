use std::fmt::Debug;

use tokio::{
    fs::OpenOptions,
    io::{self, AsyncWriteExt},
};
use tracing::error;

use crate::models::Message;

pub async fn output_to_file(filename: &str, message: &Message) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(filename)
        .await?;

    file.write_all(
        format!(
            "[{}] {} ({}): {}\n",
            message.timestamp, message.author.username, message.author.id, message.content
        )
        .as_bytes(),
    )
    .await?;

    file.flush().await?;

    Ok(())
}

pub fn log_error<T>(e: T, message: &str, fatal: bool)
where
    T: Debug,
{
    error!("{}\n", message);
    error!("{:?}", e);
    if fatal {
        std::process::exit(1);
    }
}
