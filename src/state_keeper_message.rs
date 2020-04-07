use crate::ClientMessage;
use crate::Sender;
use async_std::net::TcpStream;
use std::net::SocketAddr;
use std::sync::Arc;

pub enum StateKeeperMessage {
    NewClient {
        stream: Arc<TcpStream>,
        sender: Sender<ClientMessage>,
    },
    ClientGone {
        address: SocketAddr,
    },
    Message {
        source: SocketAddr,
        text: String,
    },
}
