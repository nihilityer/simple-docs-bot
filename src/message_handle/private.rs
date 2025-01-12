use onebot_v11::event::message::PrivateMessage;
use tracing::info;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use crate::database::DataBaseHelp;

pub async fn handle_private_message(message: PrivateMessage, _database: &DataBaseHelp) -> Result<Option<Vec<ApiPayload>>> {
    info!("Private Message Sender: {}", message.user_id);
    for msg in message.message {
        info!("Private Message: {:?}", msg);
    }
    Ok(None)
}
