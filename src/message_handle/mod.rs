use crate::database::DataBaseHelp;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::event::message::Message;

mod group;
mod private;

pub async fn handle_message(
    message: Message,
    database: &DataBaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    match message {
        Message::PrivateMessage(msg) => private::handle_private_message(msg, database).await,
        Message::GroupMessage(msg) => group::handle_group_message(msg, database).await,
    }
}
