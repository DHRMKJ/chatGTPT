use std::sync::mpsc::{channel, Sender, Receiver};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write} ;
use std::thread;

//pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, ()>;
#[derive(Debug)]
enum Message {
    ClientConnected,
    ClientDisconnected,
    NewMessage(TcpStream)
}

fn server(message_receiver: Receiver<Message>) -> Result<()> {
    loop {
        let message = message_receiver.recv().map_err(|err| {
            eprintln!("[ERROR]: error receiving message {err}");
        })?;
        println!("message: {:?}", message);
    }
    //todo!();
    Ok(())
}

fn client(mut stream: TcpStream, message_sender: Sender<Message>) -> Result<()> {
   let _ = message_sender.send(Message::ClientConnected).map_err(|err| {
        eprintln!("[ERROR]: error sending the message to server {err}");
   })?;
   let mut buff = Vec::new();
   buff.resize(64, 0);
   loop {
        stream.read(&mut buff).map_err(|err| { 
            eprintln!("[ERROR]: error reading stream message {err}");
        })?;
        message_sender.send(Message::NewMessage(stream)).map_err(|err| {
            eprintln!("[ERROR]: error sending message to the server {err}");
            message_sender.send(Message::ClientDisconnected).expect("[ERROR]: error sending message to the server");
        })?;  
    }
    Ok(())
}

fn main() -> Result<()> {
    let address = "127.0.0.1:6969";
    let listener = TcpListener::bind(address).map_err(|err|{ eprintln!("ERROR: couldn't bind {address}: {err}")})?;
    println!("listening to {address}");

    let (message_sender, message_receiver) = channel();

    thread::spawn(|| server(message_receiver));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let message_sender = message_sender.clone();
                thread::spawn(|| client(stream, message_sender));
            },
            Err(err) => {
                eprintln!("ERROR: could not appect incoming connections: {err}"); 
            }
        }
    }
    Ok(())
}
