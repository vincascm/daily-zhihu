use std::net::TcpStream;

use anyhow::{Context, Error, Result};
use http_types::{Method, Request, Url};
use serde::{Deserialize, Serialize};
use smol::Async;

const LATEST_URL: &str = "https://news-at.zhihu.com/api/3/news/latest";
const BEFORE_DATE_URL: &str = "https://news-at.zhihu.com/api/3/news/before/";

pub async fn get_latest() -> Result<Content> {
    request(LATEST_URL).await
}

pub async fn get_before_date(date: &str) -> Result<Content> {
    let url = format!("{}{}", BEFORE_DATE_URL, date);
    request(&url).await
}

async fn request<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T> {
    let req = Request::new(Method::Get, Url::parse(url)?);
    let url = req.url();
    let host = url.host().context("cannot parse host")?.to_string();
    let addr = url
        .socket_addrs(|| None)?
        .into_iter()
        .next()
        .context("invalid zhihu api url")?;
    let stream = Async::<TcpStream>::connect(addr).await?;
    let stream = async_native_tls::connect(&host, stream).await?;
    let mut resp = async_h1::connect(stream, req).await.map_err(Error::msg)?;
    if resp.status().is_success() {
        let data: T = resp.body_json().await.map_err(Error::msg)?;
        Ok(data)
    } else {
        Err(Error::msg("request zhihu api error"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    pub date: String,
    pub stories: Vec<Story>,
    pub top_stories: Option<Vec<Story>>,
}

#[derive(Serialize, Deserialize)]
pub struct Story {
    pub image_hue: Option<String>,
    pub title: String,
    pub url: String,
    pub hint: String,
    pub ga_prefix: String,
    pub images: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub _type: i32,
    pub id: i32,
}
