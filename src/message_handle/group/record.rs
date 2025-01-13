use crate::database::DatabaseHelp;
use crate::status::BotStatus;
use crate::utils::json_parse;
use crate::utils::json_parse::JsonDataType;
use anyhow::{Error, Result};
use chrono::{Duration, Local, TimeZone};
use onebot_v11::api::payload::ApiPayload;
use onebot_v11::api::payload::SendGroupMsg;
use onebot_v11::event::message::GroupMessage;
use onebot_v11::MessageSegment;
use reqwest::get;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::ops::Add;
use std::path::Path;
use tracing::{error, info, warn};

pub async fn handle_record_start(
    message: GroupMessage,
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    database.update_status(BotStatus::RecordTitle).await?;
    database.set_record_user_id(message.user_id).await?;
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
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    if !database.check_record_user_id(message.user_id).await? {
        info!("not recording user");
        return Ok(None);
    }
    if message.message.len() == 1 {
        if let MessageSegment::Text { data } = message.message[0].clone() {
            if data.text.len() > database.max_title_length().await? {
                return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
                    group_id: message.group_id,
                    message: vec![MessageSegment::text("标题也太长了，搞个短点的")],
                    auto_escape: false,
                })]));
            }
            let uuid = database.insert_new_record(data.text).await?;
            database.set_recording_uuid(uuid).await?;
            database.update_status(BotStatus::RecordContent).await?;
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
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    if !database.check_record_user_id(message.user_id).await? {
        info!("not recording user");
        return Ok(None);
    }
    let mut reply_messages = Vec::<MessageSegment>::new();
    let uuid = database.recording_uuid().await?;
    for msg in message.message {
        match msg {
            MessageSegment::Text { data } => {
                database
                    .record_content(uuid.clone(), data.text, "text".to_string())
                    .await?;
            }
            MessageSegment::Image { data } => match data.url {
                None => {
                    reply_messages.push(MessageSegment::text("图片信息获取失败:\n"));
                }
                Some(url) => {
                    let image_save_path = format!(
                        "{}/{}/{}",
                        database.share_path().await?,
                        Local::now().format("%Y-%m"),
                        uuid,
                    );
                    let save_path = Path::new(&image_save_path);
                    if !save_path.exists() {
                        create_dir_all(save_path)?;
                    }
                    let image_type = data.file.split(".").last().unwrap();
                    let save_image_name = format!("{}.{}", uuid::Uuid::new_v4(), image_type);
                    let response = get(url).await?;

                    if response.status().is_success() {
                        let mut file = File::create(save_path.join(&save_image_name))?;
                        let content = response.bytes().await?;
                        file.write_all(&content)?;
                        reply_messages.push(MessageSegment::text(format!(
                            "图片记录成功: {}\n",
                            &save_image_name
                        )));
                        database
                            .record_content(uuid.clone(), save_image_name, "image".to_string())
                            .await?;
                    } else {
                        error!("Download Image Error, Status Code: {}", response.status());
                        reply_messages.push(MessageSegment::text("图片获取失败"))
                    }
                }
            },
            MessageSegment::Json { data } => match json_parse::check_json_data_type(&data.data)? {
                JsonDataType::WeChatShare => {
                    let contents = json_parse::get_wechat_share_content(&data.data)?;
                    for content in contents {
                        database
                            .record_content(uuid.clone(), content, "text".to_string())
                            .await?;
                    }
                }
                JsonDataType::Other => {
                    warn!("Not Support Json Message: {:?}", data.data);
                    reply_messages.push(MessageSegment::text("内容解析失败，此内容暂不支持"));
                }
            },
            other => {
                warn!("Unsupported message: {:?}", other);
                reply_messages.push(MessageSegment::text("此内容暂不支持记录:\n"));
                reply_messages.push(other);
            }
        }
    }
    database.update_status(BotStatus::RecordRemark).await?;
    reply_messages.push(MessageSegment::text("内容记录完成，如果还需记录请回复：1\n当前记录者做为署名者请回复：2\n跳过署名请回复：3\n修改署名者请直接输入"));
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: reply_messages,
        auto_escape: false,
    })]))
}

pub async fn handle_record_remark(
    message: GroupMessage,
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    if !database.check_record_user_id(message.user_id).await? {
        info!("not recording user");
        return Ok(None);
    }
    let uuid = database.recording_uuid().await?;
    if let MessageSegment::Text { data } = message.message[0].clone() {
        match data.text.as_str() {
            "1" => {
                database.update_status(BotStatus::RecordContent).await?;
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
                database
                    .set_record_remark(format!("（分享者：{}）", record_user), uuid)
                    .await?;
            }
            "3" => {
                info!("recording remark skip");
            }
            record_user => {
                database
                    .set_record_remark(format!("（分享者：{}）", record_user), uuid)
                    .await?;
            }
        }
    }
    database.update_status(BotStatus::WaitingCommand).await?;
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text("记录成功！")],
        auto_escape: false,
    })]))
}

pub async fn handle_record_select(
    message: GroupMessage,
    database: &DatabaseHelp,
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
    let records = database.select_records_by_date(start, end).await?;
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
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    let admin_id = database.bot_admin().await?;
    if admin_id != message.user_id {
        warn!("Group Message Sender Error: {:?}", message.sender);
        return Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
            group_id: message.group_id,
            message: vec![MessageSegment::text("非管理员不可撤销")],
            auto_escape: false,
        })]));
    }
    let records = database.select_all_records().await?;
    let last_record = records.last().ok_or(Error::msg("last record"))?;
    let uuid = last_record.id.clone();
    database.delete_record(&uuid).await?;
    database.delete_content(&uuid).await?;
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text(format!(
            "{}\n已删除",
            last_record.title
        ))],
        auto_escape: false,
    })]))
}
