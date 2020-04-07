use async_std::io::BufReader;
use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};
use async_std::prelude::*;
use async_std::task;
use futures::channel::mpsc;
use futures::sink::SinkExt;
use std::sync::Arc;

mod client_message;
mod client_state;
mod state;
mod state_keeper_message;

use client_message::ClientMessage;
use client_state::ClientState;
use state::State;
use state_keeper_message::StateKeeperMessage;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = futures::channel::mpsc::UnboundedSender<T>;
type Receiver<T> = futures::channel::mpsc::UnboundedReceiver<T>;

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let (mut state_keeper_sender, state_keeper_receiver) = mpsc::unbounded();
    let _state_keeper = task::spawn(state_keeper_loop(state_keeper_receiver));

    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = Arc::new(stream?);
        let (client_sender, client_receiver) = mpsc::unbounded();
        let new_client = StateKeeperMessage::NewClient {
            stream: stream.clone(),
            sender: client_sender,
        };
        state_keeper_sender.send(new_client).await?;
        task::spawn(client_sender_loop(stream.clone(), client_receiver));
        task::spawn(client_receiver_loop(
            stream.clone(),
            state_keeper_sender.clone(),
        ));
    }
    Ok(())
}

async fn client_receiver_loop(
    stream: Arc<TcpStream>,
    mut state_keeper_sender: Sender<StateKeeperMessage>,
) -> Result<()> {
    let stream = &*stream;
    let address = stream.peer_addr()?;
    let mut buf_reader = BufReader::new(stream);
    loop {
        let mut text = String::new();
        let line_length = buf_reader.read_line(&mut text).await?;
        if line_length == 0 {
            break;
        }
        let message = StateKeeperMessage::Message {
            text,
            source: address,
        };
        state_keeper_sender.send(message).await?;
    }
    state_keeper_sender
        .send(StateKeeperMessage::ClientGone { address })
        .await?;
    Ok(())
}

async fn client_sender_loop(
    stream: Arc<TcpStream>,
    mut receiver: Receiver<ClientMessage>,
) -> Result<()> {
    let mut counter: i32 = 0;
    let mut stream = &*stream;

    while let Some(update) = receiver.next().await {
        match update {
            ClientMessage::StateUpdate { new_state } => {
                let s = format!("Update #{}:\n", counter);
                stream.write(s.as_bytes()).await?;
                let s = new_state.to_string();
                stream.write(s.as_bytes()).await?;
            }
            ClientMessage::Error { message } => {
                let s = format!("Error: {}", message);
                stream.write(s.as_bytes()).await?;
            }
        }

        counter += 1;
    }

    Ok(())
}

async fn state_keeper_loop(mut receiver: Receiver<StateKeeperMessage>) -> Result<()> {
    let mut state = State::new();

    while let Some(message) = receiver.next().await {
        match message {
            StateKeeperMessage::NewClient { stream, sender } => {
                state.add_client(stream.peer_addr()?, sender);
                state.distribute().await?;
            }
            StateKeeperMessage::ClientGone { address } => {
                state.remove_client(&address);
                state.distribute().await?;
            }
            StateKeeperMessage::Message { source, text } => {
                match text.trim().parse() {
                    Ok(new_number) => {
                        state.set_client_number(&source, new_number);
                        state.distribute().await?;
                    }

                    Err(e) => {
                        let message = format!("{}\n", e);
                        state.send_error_to_client(&source, &message).await?;
                    }
                };
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    task::block_on(accept_loop("127.0.0.1:12345"))?;
    Ok(())
}
