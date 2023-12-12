use std::sync::mpsc::{channel, Sender, Receiver};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write} ;
use std::collections::HashMap;
use std::sync::Arc;

extern crate chatstuff;
use chatstuff::ThreadPool;

pub type Result<T> = std::result::Result<T, ()>;

#[derive(Debug)]
struct Client {
    stream: Arc<TcpStream>
}

#[derive(Debug)]
enum Message {
    ClientConnected{author: Arc<TcpStream>},
    ClientDisconnected{author: Arc<TcpStream>},
    NewMessage{bytes:Vec<u8>, author: Arc<TcpStream>}
}


fn server(message_receiver: Receiver<Message>) -> Result<()> {
    let mut clients = HashMap::new();
    loop {
        let message = message_receiver.recv()
                        .expect("[ERROR]: could not get the message: {err}");
        match message {
            Message::ClientConnected{author} => {
                  let addr = author.as_ref().peer_addr().expect("[ERROR]: could not get the client address");
                  clients.insert(addr.clone(), Client {
                        stream: author.clone()
                  });
            },
            Message::NewMessage{author, bytes} => { 
                  let addrs = author.as_ref().peer_addr().expect("[ERROR]: could not get the client address");
                  for (addr, client) in clients.iter() {
                      if &addrs != addr {
                        client.stream.as_ref().write(&bytes).map_err(|err| {
                            eprintln!("[ERROR]: could not write on the stream {err}");
                        })?;
                      }
                  }
            },
            Message::ClientDisconnected{author} => {
                let addr = author.as_ref().peer_addr().expect("[ERROR]: could not get the client address");
                clients.remove(&addr);
            },
        }
    }
   Ok(())
}

fn client(stream: Arc<TcpStream>, message_sender: Arc<Sender<Message>>) -> Result<()> {
    message_sender.as_ref().send(Message::ClientConnected{author: Arc::clone(&stream)}).map_err(|err| eprintln!("[ERROR]: could not send message to the server {err}"))?;
    
    let mut buffer: Vec<u8> = vec![0; 64];

    loop {
        let bytes_read = stream.as_ref().read(&mut buffer[..]).map_err(|err| {
            eprintln!("[ERROR]: error reading into the client buffer {err}");
            message_sender.send(Message::ClientDisconnected{author: Arc::clone(&stream)}).expect("[ERROR]: could not disconnect client");
        })?;
        
        if bytes_read == 0 {
            message_sender.send(Message::ClientDisconnected{author: Arc::clone(&stream)}).expect("[ERROR]: could not disconnect client");
            break;
        }
        
        message_sender.as_ref().send(Message::NewMessage{author:Arc::clone(&stream), bytes: buffer.clone()}).map_err(|err| {
            eprintln!("[ERROR]: error reading into the client buffer {err}");
            message_sender.send(Message::ClientDisconnected{author: Arc::clone(&stream)}).expect("[ERROR]: could not disconnect client");
        })?; 
    }

    Ok(())
}

fn main() -> Result<()> {
    let address = "127.0.0.1:6969";
    let listener = TcpListener::bind(address).map_err(|err|{ eprintln!("ERROR: couldn't bind {address}: {err}")})?;
    println!("listening to {address}");
    let size: usize = 2000;    
    let pool = ThreadPool::new(size);
    
    let (message_sender, message_receiver) = channel();
    let message_sender = Arc::new(message_sender);

    pool.execute(|| { server(message_receiver); });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stream = Arc::new(stream);
                let message_sender = Arc::clone(&message_sender);
                pool.execute(|| {
                    client(stream, message_sender);
                })
            },
            Err(err) => {
                eprintln!("[ERROR]: could not accept incoming connections: {err}");
            }
        }
    }

    Ok(())
}
