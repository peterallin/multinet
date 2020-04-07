use crate::{ClientMessage, Sender};

#[derive(Clone)]
pub struct ClientState {
    pub number: i64,
    pub sender: Sender<ClientMessage>,
}
