use crate::database::DatabaseHelp;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::event::message::Message;
use crate::config::CoreConfig;

mod group;
mod private;

pub async fn handle_message(
    config: &CoreConfig,
    message: Message,
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    match message {
        Message::PrivateMessage(msg) => private::handle_private_message(config, msg, database).await,
        Message::GroupMessage(msg) => group::handle_group_message(msg, database).await,
    }
}
