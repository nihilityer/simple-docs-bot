use std::sync::Arc;
use crate::bot_help::BotHelp;
use anyhow::{anyhow, Result};
use onebot_v11::api::payload::{ApiPayload, GetMsg};
use onebot_v11::api::resp::{ApiRespData, GetMsgResponse};

pub async fn get_reply_original_message(
    message_id: String,
    bot_help: Arc<BotHelp>,
) -> Result<GetMsgResponse> {
    let search_result = bot_help.ws_connect.clone().call_api(ApiPayload::GetMsg(GetMsg {
        message_id: message_id.parse()?
    })).await?;
    if let ApiRespData::GetMsgResponse(resp) = search_result.data {
        return Ok(resp);
    }
    Err(anyhow!("Get Original Message Failed"))
}
