use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use anyhow::{anyhow, Result};
use reqwest::get;
use tracing::error;

pub async fn get_image(url: String, image_type: String, save_path: String) -> Result<String> {
    let save_path = Path::new(&save_path);
    if !save_path.exists() {
        create_dir_all(save_path)?;
    }
    let save_image_name = format!("{}.{}", uuid::Uuid::new_v4(), image_type);
    let response = get(url).await?;

    if response.status().is_success() {
        let mut file = File::create(save_path.join(&save_image_name))?;
        let content = response.bytes().await?;
        file.write_all(&content)?;
        
        Ok(save_image_name)
    } else {
        error!("Download Image Error, Status Code: {}", response.status());
        Err(anyhow!("Download Image Error"))
    }
} 