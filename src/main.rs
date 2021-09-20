use std::net::ToSocketAddrs;

use anyhow::{Context, Result};

mod server;
mod zhihu_api;

fn main() -> Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:3002".to_string());
    let addr = addr
        .to_socket_addrs()?
        .next()
        .context("invalid listen addr")?;
    server::listen(addr)
}
