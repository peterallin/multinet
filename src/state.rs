use crate::ClientMessage;
use crate::ClientState;
use crate::Result;
use crate::Sender;
use futures::sink::SinkExt;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct State {
    clients: HashMap<SocketAddr, ClientState>,
}

impl State {
    pub fn new() -> Self {
        State {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, address: SocketAddr, sender: Sender<ClientMessage>) {
        self.clients
            .insert(address, ClientState { sender, number: 0 });
    }

    pub fn remove_client(&mut self, address: &SocketAddr) {
        self.clients.remove(address);
    }

    pub fn set_client_number(&mut self, client_address: &SocketAddr, new_number: i64) {
        self.clients
            .entry(*client_address)
            .and_modify(|e| e.number = new_number);
    }

    pub async fn send_error_to_client(
        &mut self,
        client_address: &SocketAddr,
        message: &str,
    ) -> Result<()> {
        if let Some(client) = self.clients.get_mut(client_address) {
            let message = message.to_string();
            client.sender.send(ClientMessage::Error { message }).await?;
        }
        Ok(())
    }

    pub async fn distribute(&self) -> Result<()> {
        for client_state in self.clients.clone().values_mut() {
            client_state
                .sender
                .send(ClientMessage::StateUpdate {
                    new_state: self.clone(),
                })
                .await?;
        }
        Ok(())
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (key, value) in &self.clients {
            writeln!(f, "{} -> {}", key, value.number)?;
        }
        Ok(())
    }
}
