use crate::config::DataBaseConfig;
use crate::status::BotStatus;
use anyhow::Result;
use chrono::{DateTime, Local};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct DatabaseHelp {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
pub struct Record {
    pub id: String,
    pub title: String,
    pub remark: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(sqlx::FromRow)]
pub struct Content {
    pub uuid: String,
    pub content: String,
    pub content_type: String,
}

impl DatabaseHelp {
    pub async fn init(config: &DataBaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!(
                "postgres://{}:{}@{}:{}/{}",
                config.username, config.password, config.host, config.port, config.database
            ))
            .await?;
        Ok(DatabaseHelp { pool })
    }

    pub async fn bot_admin(&self) -> Result<i64> {
        let row: (String,) =
            sqlx::query_as("SELECT data_value FROM bot_data WHERE data_key = 'admin'")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0.parse::<i64>()?)
    }

    pub async fn bot_status(&self) -> Result<BotStatus> {
        let row: (String,) =
            sqlx::query_as("SELECT data_value FROM bot_data WHERE data_key = 'status'")
                .fetch_one(&self.pool)
                .await?;
        Ok(BotStatus::from(row.0))
    }

    pub async fn max_title_length(&self) -> Result<usize> {
        let row: (String,) =
            sqlx::query_as("SELECT data_value FROM bot_data WHERE data_key = 'max_title_length'")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0.parse::<usize>()?)
    }

    pub async fn update_status(&self, status: BotStatus) -> Result<()> {
        sqlx::query("UPDATE bot_data SET data_value = $1 WHERE data_key = 'status'")
            .bind(status.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn check_record_user_id(&self, user_id: i64) -> Result<bool> {
        let row: (String,) =
            sqlx::query_as("SELECT data_value FROM bot_data WHERE data_key = 'record_user_id'")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0.parse::<i64>()? == user_id)
    }

    pub async fn set_record_user_id(&self, user_id: i64) -> Result<()> {
        sqlx::query("UPDATE bot_data SET data_value = $1 WHERE data_key = 'record_user_id'")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn share_path(&self) -> Result<String> {
        let row: (String,) =
            sqlx::query_as("SELECT data_value FROM bot_data WHERE data_key = 'share_path'")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    pub async fn recording_uuid(&self) -> Result<String> {
        let row: (String,) =
            sqlx::query_as("SELECT data_value FROM bot_data WHERE data_key = 'recording_uuid'")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    pub async fn set_recording_uuid(&self, uuid: String) -> Result<()> {
        sqlx::query("UPDATE bot_data SET data_value = $1 WHERE data_key = 'recording_uuid'")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_new_record(&self, title: String) -> Result<String> {
        let uuid = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO records (id, title) VALUES ($1, $2)")
            .bind(&uuid)
            .bind(title)
            .execute(&self.pool)
            .await?;
        Ok(uuid)
    }

    pub async fn record_content(
        &self,
        uuid: String,
        content: String,
        content_type: String,
    ) -> Result<()> {
        sqlx::query("INSERT INTO content (uuid, content, content_type) VALUES ($1, $2, $3)")
            .bind(uuid)
            .bind(content)
            .bind(content_type)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_record_remark(&self, remark: String, uuid: String) -> Result<()> {
        sqlx::query("UPDATE records SET remark = $1 WHERE id = $2")
            .bind(remark)
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn select_all_records(&self) -> Result<Vec<Record>> {
        let rows: Vec<Record> = sqlx::query_as(
            "SELECT id, title, remark, created_at, updated_at FROM records ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn select_all_content_by_uuid(&self, uuid: String) -> Result<Vec<Content>> {
        let rows: Vec<Content> = sqlx::query_as(
            "SELECT uuid, content, content_type FROM content WHERE uuid = $1 and delete_status = false ORDER BY create_time DESC",
        )
            .bind(uuid)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
}
