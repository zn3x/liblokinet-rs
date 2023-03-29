use std::time::Duration;

use tokio::io::{AsyncWriteExt, AsyncReadExt};

use liblokinet::Context;

static SEED: &[u8] = include_bytes!("../lokinet.signed");

#[tokio::main]
async fn main() {
    let mut ctx = Context::new();

    //ctx.bootstrap_rc(SEED).await;

    ctx.start().await;

    loop {
        if ctx.status().await == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    };

    let mut stream = ctx.new_tcp_stream("dw68y1xhptqbhcm5s8aaaip6dbopykagig5q5u1za4c7pzxto77y.loki:80").await.unwrap();
    stream.write_all(b"GET / HTTP/1.1\r\nHost: dw68y1xhptqbhcm5s8aaaip6dbopykagig5q5u1za4c7pzxto77y.loki:80\r\nConnection: close\r\n\r\n").await.unwrap();

    let mut buf = [0u8; 400];

    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        stream.read(&mut buf).await;
        println!("hello {}", String::from_utf8_lossy(buf.as_slice()).to_owned());
    }
}
