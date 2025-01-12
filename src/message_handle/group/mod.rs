mod record;
mod generate;

use onebot_v11::event::message::GroupMessage;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::MessageSegment;
use tracing::{debug, info, warn};
use crate::database::DataBaseHelp;
use crate::status::BotStatus;

pub async fn handle_group_message(message: GroupMessage, database: &DataBaseHelp) -> Result<Option<Vec<ApiPayload>>> {
    info!("Recv Group Message From: {}", message.group_id);
    info!("Group Message Sender: {}", message.user_id);
    info!("Group Message Sender Info: {:?}", message.sender);
    for msg in &message.message {
        info!("Message: {:?}", msg);
    }
    debug!("Group Message: {:?}", message);
    match database.bot_status().await? {
        BotStatus::WaitingCommand => {
            if message.message.len() == 1 {
                if let MessageSegment::Text { data } = message.message[0].clone() {
                    match data.text.as_str() { 
                        "记录" | "record" | "rc" => {
                            info!("Recv Record Command");
                            return record::handle_record_start(message, database).await;
                        },
                        "生成" | "generate" | "gen" => {
                            info!("Recv Generate Command");
                            return generate::handle_generate(message, database).await;
                        }
                        _ => {}
                    }
                }
            }
        }
        BotStatus::RecordTitle => return record::handle_record_title(message, database).await,
        BotStatus::RecordContent => return record::handle_record_content(message, database).await,
        BotStatus::RecordRemark => return record::handle_record_remark(message, database).await,
        BotStatus::HandleOtherCommand => {
            warn!("HandleOtherCommand Status");
        }
    }
    Ok(None)
}
