use crate::config::GitConfig;
use anyhow::{anyhow, Result};
use chrono::Local;
use onebot_v11::api::payload::{ApiPayload, SendPrivateMsg};
use onebot_v11::MessageSegment;
use std::process::Command;
use tracing::log::{error, info};

pub async fn auto_git_task(config: &GitConfig, admin_id: i64) -> Result<Option<Vec<ApiPayload>>> {
    let dir = config.repository_dir.clone();
    let username = config.username.clone();
    let password = config.password.clone().replace("@", "%40");
    let url = config.url.clone();
    let output = Command::new("git")
        .current_dir(&dir)
        .arg("add")
        .arg(".")
        .output()?;
    if !output.status.success() {
        let err_msg = format!(
            "execute `git add .` error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        error!("{}", &err_msg);
        return Err(anyhow!(err_msg));
    }

    let output = Command::new("git")
        .current_dir(&dir)
        .arg("commit")
        .arg("-m")
        .arg(Local::now().format("%Y%m%d bot commit").to_string())
        .output()?;
    if !output.status.success() {
        let err_msg = format!(
            "execute `git commit` error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        error!("{}", &err_msg);
        return Err(anyhow!(err_msg));
    }

    let output = Command::new("git")
        .current_dir(dir)
        .arg("push")
        .arg(format!("https://{username}:{password}@{url}"))
        .output()?;
    if !output.status.success() {
        let err_msg = format!(
            "execute `git push` error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        error!("{}", &err_msg);
        return Err(anyhow!(err_msg));
    }
    info!("{}", "auto git task complete");
    Ok(Some(vec![ApiPayload::SendPrivateMsg(SendPrivateMsg {
        user_id: admin_id,
        message: vec![MessageSegment::text("git任务完成")],
        auto_escape: false,
    })]))
}

pub fn git_init(config: &GitConfig) -> Result<()> {
    let dir = format!("/docs/{}", config.repository_dir.clone());

    let output = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("--add")
        .arg("safe.directory")
        .arg(dir)
        .output()?;
    if !output.status.success() {
        let err_msg = format!(
            "set git safe.directory error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        error!("{}", &err_msg);
    }
    
    let output = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("user.name")
        .arg(config.username.clone())
        .output()?;
    if !output.status.success() {
        let err_msg = format!(
            "set git user.name error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        error!("{}", &err_msg);
    }

    let output = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("user.email")
        .arg(config.user_email.clone())
        .output()
        .expect("Failed to set git user.email");
    if !output.status.success() {
        let err_msg = format!(
            "set git user.email error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        error!("{}", &err_msg);
    }
    info!("{}", "auto set git config");
    Ok(())
}
