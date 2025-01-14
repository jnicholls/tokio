//! A proxy that forwards data to another server and forwards that server's
//! responses back to clients.
//!
//! Because the Tokio runtime uses a thread pool, each TCP connection is
//! processed concurrently with all other TCP connections across multiple
//! threads.
//!
//! You can showcase this by running this in one terminal:
//!
//!     cargo run --example proxy
//!
//! This in another terminal
//!
//!     cargo run --example echo
//!
//! And finally this in another terminal
//!
//!     cargo run --example connect 127.0.0.1:8081
//!
//! This final terminal will connect to our proxy, which will in turn connect to
//! the echo server, and you'll be able to see data flowing between them.

#![warn(rust_2018_idioms)]

use futures::{future::try_join, FutureExt, StreamExt};
use std::{env, error::Error};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = env::args().nth(1).unwrap_or("127.0.0.1:8081".to_string());
    let server_addr = env::args().nth(2).unwrap_or("127.0.0.1:8080".to_string());

    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let mut incoming = TcpListener::bind(listen_addr).await?.incoming();

    while let Some(Ok(inbound)) = incoming.next().await {
        let transfer = transfer(inbound, server_addr.clone()).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer; error={}", e);
            }
        });

        tokio::spawn(transfer);
    }

    Ok(())
}

async fn transfer(inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
    let outbound = TcpStream::connect(proxy_addr).await?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = ri.copy(&mut wo);
    let server_to_client = ro.copy(&mut wi);

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}
