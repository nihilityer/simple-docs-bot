use crate::config::CoreConfig;
use crate::database::DatabaseHelp;
use crate::status::BotStatus;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::event::message::PrivateMessage;
use onebot_v11::MessageSegment;
use tracing::{debug, info, warn};

pub async fn handle_private_message(
    config: &CoreConfig,
    message: PrivateMessage,
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    info!("Recv Private Message From: {}", message.user_id);
    info!("Private Message Sender Info: {:?}", message.sender);
    for msg in &message.message {
        info!("Message: {:?}", msg);
    }
    debug!("Private Message: {:?}", message);
    if message.message.len() == 1 {
        if let MessageSegment::Text { data } = message.message[0].clone() {
            let admin_id = database.bot_admin().await?;
            if admin_id != message.user_id {
                warn!("Private Message Sender Error: {:?}", message.sender);
                return Ok(None);
            }
            match data.text.as_str() {
                "git" => return crate::utils::git::auto_git_task(&config.git, admin_id).await,
                "reset" => {
                    database.update_status(BotStatus::WaitingCommand).await?;
                }
                _ => {}
            }
        }
    }
    Ok(None)
}
