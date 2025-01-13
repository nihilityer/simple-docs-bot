use crate::database::{DatabaseHelp, Record};
use anyhow::Result;
use onebot_v11::api::payload::{ApiPayload, SendGroupMsg};
use onebot_v11::event::message::GroupMessage;
use onebot_v11::MessageSegment;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::ops::Add;
use std::path::Path;

pub async fn handle_generate(
    message: GroupMessage,
    database: &DatabaseHelp,
) -> Result<Option<Vec<ApiPayload>>> {
    let records = database.select_all_records().await?;
    generate_by_records(message, database, records).await
}

async fn generate_by_records(
    message: GroupMessage,
    database: &DatabaseHelp,
    records: Vec<Record>,
) -> Result<Option<Vec<ApiPayload>>> {
    let mut record_order_by_month: HashMap<String, Vec<Record>> = HashMap::new();
    for record in records {
        if let Some(rcs) =
            record_order_by_month.get_mut(&record.created_at.format("%Y-%m").to_string())
        {
            rcs.push(record);
        } else {
            record_order_by_month
                .insert(record.created_at.format("%Y-%m").to_string(), vec![record]);
        }
    }
    let mut root_readme_content = String::from(
        "---
title: 分享
icon: comments
index: false
---

",
    );
    let share_path = database.share_path().await?;
    let root_path = Path::new(&share_path);
    if !root_path.exists() {
        create_dir_all(root_path)?;
    }
    for (key, value) in record_order_by_month {
        root_readme_content =
            root_readme_content.add(format!("\n\n- [{}]({}/README.md)", key, key).as_str());
        let mut key_readme_content = format!(
            "---
title: {}月分享整理
icon: circle-info
index: false
---

",
            key
        );
        let mut record_order_by_day: HashMap<String, Vec<Record>> = HashMap::new();
        for record in value {
            if let Some(rcs) =
                record_order_by_day.get_mut(&record.created_at.format("%Y-%m-%d").to_string())
            {
                rcs.push(record);
            } else {
                key_readme_content = key_readme_content.add(
                    format!(
                        "\n\n- [{}]({}.md)",
                        record.created_at.format("%Y-%m-%d"),
                        record.created_at.format("%Y-%m-%d")
                    )
                    .as_str(),
                );
                record_order_by_day.insert(
                    record.created_at.format("%Y-%m-%d").to_string(),
                    vec![record],
                );
            }
        }
        let month_path = root_path.join(format!("{}/README.md", key));
        if !month_path.parent().unwrap().exists() {
            create_dir_all(month_path.parent().unwrap())?;
        }
        let mut file = File::create(&month_path)?;

        writeln!(file, "{}", key_readme_content)?;
        for (day_key, day_value) in record_order_by_day {
            let generate_path = root_path.join(format!("{}/{}.md", key, day_key));
            let save_path = Path::new(&generate_path);
            if !save_path.parent().unwrap().exists() {
                create_dir_all(save_path.parent().unwrap())?;
            }
            let mut file = File::create(save_path)?;
            writeln!(
                file,
                "---\ntitle: {}日分享整理\nicon: circle-info\n---\n",
                day_value[0].created_at.format("%Y年%m月%d")
            )?;
            for record in day_value {
                writeln!(file, "## {}\n", record.title)?;
                if let Some(remark) = record.remark {
                    writeln!(file, "{}\n", remark)?;
                }
                let contents = database.select_all_content_by_uuid(&record.id).await?;
                for content in contents {
                    match content.content_type.as_str() {
                        "text" => writeln!(file, "{}\n", content.content)?,
                        "image" => {
                            writeln!(file, "![image]({}/{})\n", content.uuid, content.content)?
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let generate_path = root_path.join("README.md");
    let mut file = File::create(&generate_path)?;
    writeln!(file, "{}", root_readme_content)?;
    Ok(Some(vec![ApiPayload::SendGroupMsg(SendGroupMsg {
        group_id: message.group_id,
        message: vec![MessageSegment::text("记录文件生成成功")],
        auto_escape: false,
    })]))
}
