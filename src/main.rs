use crate::config::CoreConfig;
use crate::database::DatabaseHelp;
use crate::log::Log;
use anyhow::Result;
use onebot_v11::api::payload::{ApiPayload, SendPrivateMsg};
use onebot_v11::connect::ws::WsConnect;
use onebot_v11::Event::*;
use onebot_v11::MessageSegment;
use tracing::{debug, error};

mod config;
mod database;
mod log;
mod message_handle;
mod status;
pub(crate) mod utils;

#[tokio::main]
pub async fn main() -> Result<()> {
    let config = CoreConfig::init()?;
    Log::init(&config.log)?;
    let database = DatabaseHelp::init(&config.data_base).await?;
    let ws_connect = WsConnect::new(config.bot_ws.clone()).await?;
    let mut receiver = ws_connect.subscribe().await;

    utils::git::git_init(&config.git)?;

    loop {
        match receiver.recv().await? {
            Message(message) => {
                match message_handle::handle_message(&config, message, &database).await {
                    Ok(payload_option) => {
                        if let Some(api_payloads) = payload_option {
                            for payload in api_payloads {
                                ws_connect.clone().call_api(payload).await?;
                            }
                        }
                    }
                    Err(error) => {
                        error!("{:?}", error);
                        let user_id = database.bot_admin().await?;
                        let text_warn = MessageSegment::text("消息处理出现问题，请及时处理");
                        ws_connect
                            .clone()
                            .call_api(ApiPayload::SendPrivateMsg(SendPrivateMsg {
                                user_id,
                                message: vec![text_warn],
                                auto_escape: false,
                            }))
                            .await?;
                    }
                }
            }
            Meta(meta) => {
                debug!("meta: {:?}", meta);
            }
            Notice(notice) => {
                debug!("notice: {:?}", notice);
            }
            Request(request) => {
                debug!("request: {:?}", request);
            }
            ApiRespBuilder(resp) => {
                debug!("resp: {:?}", resp);
            }
        }
    }
}
