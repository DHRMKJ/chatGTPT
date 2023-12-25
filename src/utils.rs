pub type Result<T> = std::result::Result<T, ()>;

#[derive(Debug)]
pub enum Message {
    ClientConnected{
        stream: std::sync::Arc<tokio::net::TcpStream>,
        addr: std::net::SocketAddr
    },
    ClientDisconnected (std::net::SocketAddr),
    NewMessage{message: Vec<u8>, addr: std::net::SocketAddr},
}
