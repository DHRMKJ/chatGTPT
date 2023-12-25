use tokio::sync::mpsc;
use tokio::net::TcpStream;
use std::sync::Arc;
use std::io;
use crate::utils::*;


pub async fn client(stream: (TcpStream, std::net::SocketAddr), message_sender: Arc<mpsc::Sender<Message>>) -> Result<()> {
    let (stream, addr) = stream;
    let stream = Arc::new(stream);
    let mut conn = true;

    match message_sender
            .as_ref()
            .send(Message::ClientConnected{stream: Arc::clone(&stream), addr})
            .await {
                Ok(_) => {
                    println!("{addr} joined");
                },
                Err(e) => {
                    eprintln!("[ERROR]: could not connect client {e}");
                    conn = false;
                }
            }

    match stream.as_ref().readable().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[ERROR]: stream is not readable");
            conn = false;
        }
    }

    while conn == true {
            let mut buffer: Vec<u8> = vec![0; 1024];
            match stream.as_ref().try_read(&mut buffer) {
                Ok(0) => {
                    println!("disconnected");
                    let _ = message_sender
                        .send(Message::ClientDisconnected(addr))
                        .await;
                    conn = false;
                },
                Ok(n) => {
                    let addr = addr.clone();
                    match message_sender.send(Message::NewMessage{message: buffer.clone(), addr})
                        .await {
                        Ok(_) => {
                            println!("sent message {addr}");
                        }
                        Err(e) => {
                            message_sender.send(Message::ClientDisconnected(addr)).await;
                            conn = false;
                        }
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(0)).await;
                    continue;
                }
                Err(e) => {
                    message_sender.send(Message::ClientDisconnected(addr)).await;
                    eprintln!("[ERROR]: error writing into the stream {e}");
                    conn = false;
                }
            }
    }
    Ok(())
}
