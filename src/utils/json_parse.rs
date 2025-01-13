use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use url::Url;

pub enum JsonDataType {
    WeChatShare,
    Other,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WeChatShare {
    pub app: String,
    pub bizsrc: String,
    pub config: WeChatShareConfig,
    pub extra: WeChatShareExtra,
    pub meta: WeChatShareMeta,
    pub prompt: String,
    pub ver: String,
    pub view: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WeChatShareConfig {
    pub ctime: usize,
    pub forward: usize,
    pub token: String,
    #[serde(rename = "type")]
    pub config_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WeChatShareExtra {
    pub app_type: usize,
    pub appid: usize,
    pub msg_seq: usize,
    pub uin: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WeChatShareMeta {
    pub news: WeChatShareMetaNews,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WeChatShareMetaNews {
    pub app_type: usize,
    pub appid: usize,
    pub ctime: usize,
    pub desc: String,
    #[serde(rename = "jumpUrl")]
    pub jump_url: String,
    pub preview: String,
    pub tag: String,
    #[serde(rename = "tagIcon")]
    pub tag_icon: String,
    pub title: String,
    pub uin: usize,
}

pub fn check_json_data_type(json: &String) -> Result<JsonDataType> {
    if serde_json::from_str::<WeChatShare>(&json).is_ok() {
        return Ok(JsonDataType::WeChatShare);
    }
    Ok(JsonDataType::Other)
}

pub fn get_wechat_share_content(json: &String) -> Result<Vec<String>> {
    let data = serde_json::from_str::<WeChatShare>(&json)?.meta.news;
    Ok(vec![
        data.title,
        get_short_wechat_share_url(&data.jump_url)?,
    ])
}

fn get_short_wechat_share_url(url_str: &String) -> Result<String> {
    let url = Url::parse(url_str.as_str())?;
    let biz = url
        .query_pairs()
        .find(|(key, _)| key == "__biz")
        .ok_or(Error::msg("No __biz"))?
        .1;
    let mid = url
        .query_pairs()
        .find(|(key, _)| key == "mid")
        .ok_or(Error::msg("No mid"))?
        .1;
    let idx = url
        .query_pairs()
        .find(|(key, _)| key == "idx")
        .ok_or(Error::msg("No idx"))?
        .1;
    let sn = url
        .query_pairs()
        .find(|(key, _)| key == "sn")
        .ok_or(Error::msg("No sn"))?
        .1;
    Ok(format!(
        "https://mp.weixin.qq.com/s?__biz={}&mid={}&idx={}&sn={}",
        biz, mid, idx, sn
    ))
}
