use std::net::{SocketAddr, TcpListener};

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use http_types::{
    mime::{CSS, HTML, ICO, PLAIN, PNG},
    Mime, Request, Response, StatusCode,
};
use serde::Serialize;
use smol::{block_on, spawn, Async};

use crate::zhihu_api::{get_before_date, get_latest, Content};

pub fn listen(addr: SocketAddr) -> Result<()> {
    block_on(async {
        let listener = Async::<TcpListener>::bind(addr)?;
        loop {
            let (stream, _) = listener.accept().await?;
            let stream = async_dup::Arc::new(stream);
            spawn(async move {
                if let Err(err) = async_h1::accept(stream, serve).await {
                    println!("Connection error: {:#?}", err);
                }
            })
            .detach();
        }
    })
}

fn response_asset(mime: Mime, asset: &[u8]) -> Response {
    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(mime);
    res.set_body(asset);
    res
}

async fn render_content(content: Content) -> Result<Response> {
    let string = minijinja::Environment::new().render_str(
        include_str!("../templates/index.html"),
        minijinja::context!(
            content => content,
            cal_date => CalDate::new(NaiveDate::parse_from_str(&content.date, "%Y%m%d")?),
        ),
    )?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(HTML);
    res.set_body(string);
    Ok(res)
}

async fn serve(req: Request) -> http_types::Result<Response> {
    let url = req.url();
    let path = url.path();
    Ok(match path {
        "/" => {
            let content = get_latest().await?;
            render_content(content).await?
        }
        "/favicon.ico" => response_asset(ICO, &include_bytes!("../asset/favicon.ico")[..]),
        "/logo.png" => response_asset(PNG, &include_bytes!("../asset/logo.png")[..]),
        "/main.css" => response_asset(CSS, &include_bytes!("../asset/main.css")[..]),
        x => {
            if x.starts_with("/before/") {
                let date_str = x
                    .strip_prefix("/before/")
                    .context("invalid before url format")?;
                let content = get_before_date(date_str).await?;
                render_content(content).await?
            } else {
                let mut res = Response::new(StatusCode::NotFound);
                res.set_content_type(PLAIN);
                res
            }
        }
    })
}

#[derive(Serialize)]
struct CalDate {
    day: String,
    month: String,
}

impl CalDate {
    fn new(date: NaiveDate) -> Self {
        Self {
            day: date.day().to_string(),
            month: format!("{}月", Self::month_name(date.month())),
        }
    }

    fn month_name(month: u32) -> &'static str {
        match month {
            2 => "二",
            3 => "三",
            4 => "四",
            5 => "五",
            6 => "六",
            7 => "七",
            8 => "八",
            9 => "九",
            10 => "十",
            11 => "十一",
            12 => "十二",
            _ => "一",
        }
    }
}
