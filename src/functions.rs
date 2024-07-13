use std::time::Duration;
use tokio::time;
use tracing::info;

use crate::{
    models::{Message, State, User},
    utils::{log_error, output_to_file},
};

pub async fn fetch_current_user_id(client: &reqwest::Client, state: &State) -> String {
    let res = client
        .get("https://discord.com/api/v10/users/@me")
        .header("Authorization", &state.config.user_token)
        .send()
        .await
        .unwrap_or_else(|e| {
            log_error(
                e,
                "Failed to fetch the user. Please check your network connection.",
                true,
            );
            unreachable!();
        });

    if let Err(e) = res.error_for_status_ref() {
        log_error(
            e,
            "Failed to fetch the user. Please check your token and try again.",
            true,
        );
    }

    let user = res.json::<User>().await.unwrap_or_else(|e| {
        log_error(e, "Failed to parse the user.", true);
        unreachable!();
    });

    info!("Logged in as {}", user.username);

    user.id
}

pub async fn start(client: &reqwest::Client, state: &mut State) {
    let mut last_message_id = state.args.before_id.clone();

    info!("Fetching messages in {}", state.args.channel_id);

    loop {
        let mut url = format!(
            "https://discord.com/api/v10/channels/{}/messages?limit=100",
            state.args.channel_id,
        );

        if let Some(ref id) = last_message_id {
            url.push_str(&format!("&before={}", id));
        }

        let res = match client
            .get(url)
            .header("Authorization", &state.config.user_token)
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                log_error(
                    e,
                    "Failed to fetch the messages. Trying again in a few seconds...",
                    false,
                );
                time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        if let Err(e) = res.error_for_status_ref() {
            log_error(
                e,
                "Failed to fetch the messages. Trying again in a few seconds...",
                false,
            );
            time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        let messages = match res.json::<Vec<Message>>().await {
            Ok(messages) => messages,
            Err(e) => {
                log_error(
                    e,
                    "Failed to parse the messages. Trying again in a few seconds...",
                    false,
                );
                time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        if messages.len() == 0 {
            break;
        }

        last_message_id = Some(messages.last().unwrap().id.to_string());

        let user_id = state.user_id.as_ref().unwrap();

        let filtered_messages: Vec<&Message> =
            messages
                .iter()
                .filter(|msg| {
                    msg.author.id == *user_id
                        || (state.args.filter.iter().any(|filter| {
                            msg.content.to_lowercase().contains(&filter.to_lowercase())
                        }))
                })
                .collect();

        info!("Fetched {} messages", messages.len());

        if filtered_messages.len() == 0 {
            time::sleep(Duration::from_millis(state.args.fetch_delay.into())).await;
            continue;
        }

        delete_messages(client, &filtered_messages, state).await;

        time::sleep(Duration::from_millis(state.args.fetch_delay.into())).await;
    }

    info!("Finished deleting messages");
}

async fn delete_messages(client: &reqwest::Client, messages: &Vec<&Message>, state: &mut State) {
    for message in messages {
        if let Some(ref id) = state.args.after_id {
            if message.id == *id {
                info!("Reached the message specified by --after-id, exiting...");
                std::process::exit(0);
            }
        }

        if state.args.only_attachments && message.attachments.len() == 0 {
            continue;
        }

        loop {
            let res = match client
                .delete(format!(
                    "https://discord.com/api/v10/channels/{}/messages/{}",
                    state.args.channel_id, message.id
                ))
                .header("Authorization", &state.config.user_token)
                .send()
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    log_error(
                        e,
                        "Failed to delete the message. Trying again in a few seconds...",
                        false,
                    );
                    time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            if let Err(e) = res.error_for_status_ref() {
                log_error(
                    e,
                    "Failed to delete the message. Trying again in a few seconds...",
                    false,
                );
                time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            state.deleted_count += 1;
            info!(
                "Deleted message {} (id: {})",
                state.deleted_count, message.id
            );

            if let Some(ref filename) = state.args.output {
                output_to_file(&filename, message)
                    .await
                    .unwrap_or_else(|e| {
                        log_error(
                            e,
                            &format!(
                                "Error writing to output file {} for message {}",
                                filename, message.id
                            ),
                            false,
                        )
                    })
            }

            if let Some(x) = state.args.number {
                if state.deleted_count >= x {
                    info!(
                        "Successfully deleted the number of messages specified by --number (-n), exiting..."
                    );
                    std::process::exit(0);
                }
            }

            time::sleep(Duration::from_millis(state.args.delete_delay.into())).await;

            break;
        }
    }
}
