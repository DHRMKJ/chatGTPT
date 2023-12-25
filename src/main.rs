mod client;
mod server;
mod utils;

use tokio::sync::mpsc;
use tokio::net::{TcpListener};
use std::sync::Arc;
use crate::utils::*;
use crate::server::*;
use crate::client::*;

#[tokio::main]
async fn main() -> Result<()> {
    let address = "127.0.0.1:6969";
    let listener = TcpListener::bind(address).await.map_err(|err| {
        eprintln!("[ERROR]: error listening in port, {err}");
    })?;

    println!("listening to address: {address}");

    let (message_sender, message_reciever) = mpsc::channel::<Message>(1000);
    let message_sender = Arc::new(message_sender);
    
    tokio::spawn(async move { 
        server(message_reciever).await;
    });

    loop {
        match listener.accept().await {
            Ok(stream) => {
                let message_sender = Arc::clone(&message_sender);
                tokio::spawn(async move {
                    client(stream, message_sender).await;
                });
            }, 
            Err(err) => eprintln!("[ERROR]: error listening to port, {err}")
        }
    }
}
