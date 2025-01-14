use std::sync::Arc;
use crate::bot_help::BotHelp;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::event::message::Message;
use crate::config::CoreConfig;

mod group;
mod private;

pub async fn handle_message(
    config: &CoreConfig,
    message: Message,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    match message {
        Message::PrivateMessage(msg) => private::handle_private_message(config, msg, bot_help).await,
        Message::GroupMessage(msg) => group::handle_group_message(msg, bot_help).await,
    }
}
