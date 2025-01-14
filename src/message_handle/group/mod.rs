mod generate;
mod record;

use std::sync::Arc;
use crate::bot_help::BotHelp;
use crate::status::BotStatus;
use anyhow::Result;
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::event::message::GroupMessage;
use onebot_v11::MessageSegment;
use tracing::{debug, info, warn};

pub async fn handle_group_message(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    info!("Recv Group Message From: {}", message.group_id);
    info!("Group Message Sender: {}", message.user_id);
    info!("Group Message Sender Info: {:?}", message.sender);
    for msg in &message.message {
        info!("Message: {:?}", msg);
    }
    debug!("Group Message: {:?}", message);
    match bot_help.bot_status().await? {
        BotStatus::WaitingCommand => {
            if message.message.len() == 2 {
                if let MessageSegment::At { data } = message.message[0].clone() {
                    if data.qq != message.self_id.to_string() {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
                if let MessageSegment::Text { data } = message.message[1].clone() {
                    match data.text.trim() {
                        "记录" | "record" | "rc" => {
                            info!("Recv Record Command");
                            return record::handle_record_start(message, bot_help).await;
                        }
                        "生成" | "generate" | "gen" => {
                            info!("Recv Generate Command");
                            return generate::handle_generate(message, bot_help).await;
                        }
                        "已记录" | "list" | "ls" => {
                            info!("Recv List Command");
                            return record::handle_record_select(message, bot_help).await;
                        }
                        "撤销" | "undo" => {
                            info!("Recv Undo Command");
                            return record::handle_record_undo(message, bot_help).await;
                        }
                        _ => {}
                    }
                }
            } else if message.message.len() == 3 {
                if let MessageSegment::At { data } = message.message[1].clone() {
                    if data.qq != message.self_id.to_string() {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
                let reply_message_id_option =
                    if let MessageSegment::Reply { data } = message.message[0].clone() {
                        Some(data.id)
                    } else {
                        None
                    };
                let command_option = if let MessageSegment::Text { data } = message.message[2].clone() {
                    Some(data.text)
                } else {
                    None
                };
                if let (Some(reply_message_id), Some(command)) = (reply_message_id_option, command_option) {
                    match command.trim() {
                        "记录" | "record" | "rc" => {
                            info!("Recv Reply Record Command");
                            return record::handle_reply_record(message.user_id, message.group_id, reply_message_id, bot_help).await;
                        }
                        _ => {}
                    }
                }
            }
        }
        BotStatus::RecordTitle => return record::handle_record_title(message, bot_help).await,
        BotStatus::RecordContent => return record::handle_record_content(message, bot_help).await,
        BotStatus::RecordRemark => return record::handle_record_remark(message, bot_help).await,
        BotStatus::HandleOtherCommand => {
            warn!("HandleOtherCommand Status");
        }
    }
    Ok(None)
}
