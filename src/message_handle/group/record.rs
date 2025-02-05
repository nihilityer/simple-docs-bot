use crate::bot_help::BotHelp;
use crate::status::BotStatus;
use crate::utils::json_parse;
use crate::utils::json_parse::JsonDataType;
use crate::utils::image;
use crate::utils::reply_message;
use anyhow::{Error, Result};
use chrono::{Duration, Local, TimeZone};
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::api::payload::SendGroupMsg;
use onebot_v11::event::message::GroupMessage;
use onebot_v11::MessageSegment;
use std::ops::Add;
use std::sync::Arc;
use onebot_v11::message::segment::{ImageData, JsonData, TextData};
use tracing::{error, info, warn};

static COMPLETE_CONTENT_RECORD_REPLY: &str = "内容记录完成，如果还需记录请回复：1\n当前记录者做为署名者请回复：2\n跳过署名请回复：3\n修改署名者请直接输入";

pub async fn handle_record_start(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    bot_help.update_status(BotStatus::RecordTitle).await?;
    bot_help.set_record_user_id(message.user_id).await?;
    let group_id = message.group_id;
    let at_message = MessageSegment::at(message.user_id.to_string());
    let text_message = MessageSegment::text("已收到记录指令，请输入标题");
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id,
        message: vec![at_message, text_message],
        auto_escape: false,
    })]))
}

pub async fn handle_record_title(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    if !bot_help.check_record_user_id(message.user_id).await? {
        info!("not recording user");
        return Ok(None);
    }
    if message.message.len() == 1 {
        if let MessageSegment::Text { data } = message.message[0].clone() {
            if data.text.len() > bot_help.max_title_length().await? {
                return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
                    group_id: message.group_id,
                    message: vec![MessageSegment::text("标题也太长了，搞个短点的")],
                    auto_escape: false,
                })]));
            }
            let uuid = bot_help.insert_new_record(data.text).await?;
            bot_help.set_recording_uuid(uuid).await?;
            bot_help.update_status(BotStatus::RecordContent).await?;
            let at_message = MessageSegment::at(message.user_id.to_string());
            let text_message = MessageSegment::text("标题记录成功，请输入内容");
            return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
                group_id: message.group_id,
                message: vec![at_message, text_message],
                auto_escape: false,
            })]));
        }
    }
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text("标题只接受纯文本")],
        auto_escape: false,
    })]))
}

pub async fn handle_record_content(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    if !bot_help.check_record_user_id(message.user_id).await? {
        info!("not recording user");
        return Ok(None);
    }
    let mut reply_messages = Vec::<MessageSegment>::new();
    let uuid = bot_help.recording_uuid().await?;
    handle_record_message_list_content(&message, &bot_help, &mut reply_messages, &uuid).await?;
    bot_help.update_status(BotStatus::RecordRemark).await?;
    reply_messages.push(MessageSegment::text(COMPLETE_CONTENT_RECORD_REPLY));
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: reply_messages,
        auto_escape: false,
    })]))
}

async fn handle_record_message_list_content(message: &GroupMessage, bot_help: &Arc<BotHelp>, reply_messages: &mut Vec<MessageSegment>, uuid: &String) -> Result<(), Error> {
    for msg in message.message.iter() {
        match msg {
            MessageSegment::Text { data } => handle_text_content(bot_help, uuid, data.clone()).await?,
            MessageSegment::Image { data } => handle_image_content(bot_help, reply_messages, uuid, data.clone()).await?,
            MessageSegment::Json { data } => handle_json_content(bot_help, reply_messages, uuid, data).await?,
            other => {
                warn!("Unsupported message: {:?}", other);
                reply_messages.push(MessageSegment::text("此内容暂不支持记录:\n"));
                reply_messages.push(other.clone());
            }
        }
    }
    Ok(())
}

async fn handle_text_content(bot_help: &Arc<BotHelp>, uuid: &str, data: TextData) -> Result<(), Error> {
    bot_help
        .record_content(uuid.to_string(), data.text, "text".to_string())
        .await?;
    Ok(())
}

async fn handle_json_content(bot_help: &Arc<BotHelp>, reply_messages: &mut Vec<MessageSegment>, uuid: &str, data: &JsonData) -> Result<(), Error> {
    match json_parse::check_json_data_type(&data.data)? {
        JsonDataType::WeChatShare => {
            let contents = json_parse::get_wechat_share_content(&data.data)?;
            for content in contents {
                bot_help
                    .record_content(uuid.to_string(), content, "text".to_string())
                    .await?;
            }
        }
        JsonDataType::Other => {
            warn!("Not Support Json Message: {:?}", data.data);
            reply_messages.push(MessageSegment::text("内容解析失败，此内容暂不支持"));
        }
    }
    Ok(())
}

async fn handle_image_content(bot_help: &Arc<BotHelp>, reply_messages: &mut Vec<MessageSegment>, uuid: &String, data: ImageData) -> Result<(), Error> {
    match data.url {
        None => {
            reply_messages.push(MessageSegment::text("图片信息获取失败:\n"));
        }
        Some(url) => {
            let image_save_path = format!(
                "{}/{}/{}",
                bot_help.share_path().await?,
                Local::now().format("%Y-%m"),
                uuid,
            );
            let image_type = data.file.split(".").last().unwrap();
            match image::get_image(url, image_type.to_string(), image_save_path).await {
                Ok(save_image_name) => {
                    reply_messages.push(MessageSegment::text(format!(
                        "图片记录成功: {}\n",
                        &save_image_name
                    )));
                    bot_help
                        .record_content(uuid.clone(), save_image_name, "image".to_string())
                        .await?;
                },
                Err(e) => {
                    error!("Image Get Error: {}", e);
                    reply_messages.push(MessageSegment::text("图片获取失败"))
                }
            };
        }
    }
    Ok(())
}

pub async fn handle_record_remark(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    if !bot_help.check_record_user_id(message.user_id).await? {
        info!("not recording user");
        return Ok(None);
    }
    let uuid = bot_help.recording_uuid().await?;
    if let MessageSegment::Text { data } = message.message[0].clone() {
        match data.text.as_str() {
            "1" => {
                bot_help.update_status(BotStatus::RecordContent).await?;
                return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
                    group_id: message.group_id,
                    message: vec![MessageSegment::text("请继续回复记录内容")],
                    auto_escape: false,
                })]));
            }
            "2" => {
                let record_user = if let Some(nickname) = message.sender.nickname {
                    nickname
                } else {
                    message.user_id.to_string()
                };
                bot_help
                    .set_record_remark(format!("（分享者：{}）", record_user), uuid)
                    .await?;
            }
            "3" => {
                info!("recording remark skip");
            }
            record_user => {
                bot_help
                    .set_record_remark(format!("（分享者：{}）", record_user), uuid)
                    .await?;
            }
        }
    }
    bot_help.update_status(BotStatus::WaitingCommand).await?;
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text("记录成功！")],
        auto_escape: false,
    })]))
}

pub async fn handle_record_select(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    let now = Local::now();
    let start_native = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or(Error::msg("start date get error"))?;
    let end_native = (now.date_naive() + Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .ok_or(Error::msg("end date get error"))?;
    let start = Local
        .from_local_datetime(&start_native)
        .single()
        .ok_or(Error::msg("single datetime get error"))?;
    let end = Local
        .from_local_datetime(&end_native)
        .single()
        .ok_or(Error::msg("single datetime get error"))?;
    let records = bot_help.select_records_by_date(start, end).await?;
    let mut reply_text = String::from("今日已记录：\n");
    for record in records {
        reply_text = reply_text.add(record.title.as_str()).add("\n");
    }
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text(reply_text)],
        auto_escape: false,
    })]))
}

pub async fn handle_record_undo(
    message: GroupMessage,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    let admin_id = bot_help.bot_admin().await?;
    if admin_id != message.user_id {
        warn!("Group Message Sender Error: {:?}", message.sender);
        return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
            group_id: message.group_id,
            message: vec![MessageSegment::text("非管理员不可撤销")],
            auto_escape: false,
        })]));
    }
    let records = bot_help.select_all_records().await?;
    let last_record = records.last().ok_or(Error::msg("last record"))?;
    let uuid = last_record.id.clone();
    bot_help.delete_record(&uuid).await?;
    bot_help.delete_content(&uuid).await?;
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text(format!(
            "{}\n已删除",
            last_record.title
        ))],
        auto_escape: false,
    })]))
}

pub async fn handle_reply_record(
    record_user: i64,
    group_id: i64,
    message_id: String,
    bot_help: Arc<BotHelp>,
) -> Result<Option<Vec<ApiPayload>>> {
    let original_message =
        reply_message::get_reply_original_message(message_id, bot_help.clone()).await?;
    if original_message.message.len() == 1 {
        match original_message.message[0].clone() {
            MessageSegment::Text { data } => {
                bot_help.set_tmp_content(data.text).await?;
            }
            MessageSegment::Json { data } => match json_parse::check_json_data_type(&data.data)? {
                JsonDataType::WeChatShare => {
                    let contents = json_parse::get_wechat_share_content(&data.data)?;
                    let uuid = bot_help.insert_new_record(contents[0].clone()).await?;
                    bot_help.set_record_user_id(record_user).await?;
                    bot_help.set_recording_uuid(uuid.clone()).await?;
                    bot_help
                        .record_content(uuid.clone(), contents[1].clone(), "text".to_string())
                        .await?;
                    bot_help.set_recording_uuid(uuid).await?;
                    bot_help.update_status(BotStatus::RecordRemark).await?;
                    return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
                        group_id,
                        message: vec![MessageSegment::text(COMPLETE_CONTENT_RECORD_REPLY)],
                        auto_escape: false,
                    })]))
                }
                JsonDataType::Other => {
                    warn!("Not Support Json Message: {:?}", data.data);
                    return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
                        group_id,
                        message: vec![MessageSegment::text("此内容暂不支持解析")],
                        auto_escape: false,
                    })]));
                }
            },
            other => {
                warn!("Unexpected message segment: {:?}", other);
                return Ok(None);
            }
        }
    }
    Ok(None)
}
