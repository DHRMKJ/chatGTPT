use tokio::sync::mpsc;
use std::io;
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::utils::*;

pub async fn server(mut message_reciever: mpsc::Receiver<Message>) {
    let mut clients:HashMap<SocketAddr, Client> = HashMap::new();
    let mut banned_mfs:HashMap<SocketAddr, Client> = HashMap::new();
    let bad_words = fill_banned_words();
    loop {
        let mut client_ok = true;
        let Some(message) = message_reciever.recv().await else { continue; };
        match message {
            Message::ClientConnected{stream, addr} => {
                if banned_mfs.contains_key(&addr) {
                    println!("Banned mf attempted to connect");
                    match banned_mfs.get(&addr) {
                        Some(mf) => {
                            let mut writable = false;
                            while !writable {
                                match mf.stream.as_ref().try_write(b"You're Banned MF!\n") {
                                    Ok(_) => { writable = true;},
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                    Err(e) => {
                                        eprintln!("[ERROR]: cant write on stream {e}");
                                        writable = true;
                                    }
                                }
                            }
                        },
                        None => {}
                    }
                    clients.remove(&addr);
                    continue;
                }
                match stream.writable().await {
                    Ok(_) => {
                        let mut writable = false;
                        while !writable {
                            match stream.as_ref().try_write(b"Welcome to the Club\n") {
                                Ok(_) => { writable = true;},
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                Err(e) => {
                                    eprintln!("[ERROR]: cant write on stream {e}");
                                    writable = true;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR]: stream not writable {e}");
                        client_ok = false;
                    }
                }
                if !client_ok {
                    return;
                }
                clients.insert(addr,  Client {
                    stream,
                    addr,
                    warnings: 0
                });
            }
            Message::NewMessage{message, addr} => {
                if banned_mfs.contains_key(&addr) {
                    println!("Banned mf attempted to connect");
                    match banned_mfs.get(&addr) {
                        Some(mf) => {
                            let mut writable = false;
                            while !writable {
                                match mf.stream.as_ref().try_write(b"You're Banned MF!\n") {
                                    Ok(_) => { writable = true;},
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                    Err(e) => {
                                        eprintln!("[ERROR]: cant write on stream {e}");
                                        writable = true;
                                    }
                                }
                            }
                        },
                        None => {}
                    }
                    clients.remove(&addr);
                    continue;
                }
                let bad_boy: bool = validate_message(&message, &bad_words);
                if bad_boy {
                    match clients.get_mut(&addr) {
                        Some(boy) => {
                            boy.warnings += 1;
                            if boy.warnings == 3 {
                                banned_mfs.insert(
                                    addr,
                                    Client {
                                        stream: boy.stream.clone(),
                                        warnings: 3,
                                        addr: addr.clone()
                                    }
                                );
                            }
                            match boy.stream.as_ref().writable().await {
                                Ok(_) => {
                                    let mut written = false;
                                    while !written {
                                        match boy.stream
                                            .as_ref()
                                            .try_write(
                                                &format!("You used Banned word, this is your {}th warning, 3 warnings and you'll be banned", boy.warnings).as_bytes()
                                            )
                                        {
                                            Ok(_) => { written = true },
                                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {},
                                            Err(e) => {
                                                eprintln!("[ERROR]: could not write in stream {e}");
                                                written = true;
                                            }
                                        }
                                    }
                                },
                                Err(err) => {
                                    eprintln!("[ERROR]: could not write in stream {err}");
                                }
                            }
                        }
                        None => {}
                    }
                } else {
                    for (addrs, client) in clients.iter() {
                        if &addr != addrs && !banned_mfs.contains_key(addrs) {
                            let Ok(_avail_size) = client.stream.writable().await else { continue; };
                            let mut written = false;
                            while !written {
                                match client.stream.as_ref().try_write(&message) {
                                    Ok(_n) => {
                                        written = true;
                                    }
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                                    Err(e) => {
                                        eprintln!("[ERROR]: cant write stream {e}");
                                        written = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Message::ClientDisconnected(addr) => {
                clients.remove(&addr);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }
}

