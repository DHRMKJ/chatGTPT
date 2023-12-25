use tokio::sync::mpsc;
use std::io;
use std::collections::HashMap;
use crate::utils::Message;

pub async fn server(mut message_reciever: mpsc::Receiver<Message>) {
    let mut clients = HashMap::new();
    loop { 
        let Some(message) = message_reciever.recv().await else { continue; };
        match message {
            Message::ClientConnected{stream, addr} => {
                clients.insert(addr, stream);
            }
            Message::NewMessage{message, addr} => {
                for (addrs, stream) in clients.iter() {
                    if &addr != addrs {
                        let Ok(_avail_size) = stream.writable().await else { continue; };
                        match stream.as_ref().try_write(&message) {
                            Ok(_n) => {}
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            Err(e) => { eprintln!("[ERROR]: cant write stream {e}"); }
                        }         
                    }
                }
            }
            Message::ClientDisconnected(addr) => {
                clients.remove(&addr);
            }
        }
    }
}

