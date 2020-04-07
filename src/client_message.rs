use crate::State;

pub enum ClientMessage {
    StateUpdate { new_state: State },
    Error { message: String },
}
