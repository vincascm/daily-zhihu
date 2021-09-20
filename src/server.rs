use std::net::{SocketAddr, TcpListener};

use anyhow::{Context, Result};
use chrono::NaiveDate;
use http_types::{
    mime::{CSS, HTML, ICO, PLAIN, PNG},
    Mime, Request, Response, StatusCode,
};
use smol::{block_on, spawn, Async};
use tera::Tera;

use crate::zhihu_api::{get_before_date, get_latest, Content};

fn response_asset(mime: Mime, asset: &[u8]) -> Response {
    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(mime);
    res.set_body(asset);
    res
}

async fn render_content(content: Content) -> Result<Response> {
    use tera::Context;

    let mut context = Context::new();
    context.insert("content", &content);
    let cal_date = NaiveDate::parse_from_str(&content.date, "%Y%m%d")?;
    let cal_date = cal_date.format("%m月%d日").to_string();
    context.insert("cal_date", &cal_date);
    let string = Tera::one_off(include_str!("../templates/index.html"), &context, false)?;
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
