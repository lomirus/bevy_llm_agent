use std::sync::{Mutex, mpsc::Receiver};

use crate::AgentMessageDelta;

#[derive(Default)]
pub(crate) enum AgentStatus {
    #[default]
    Idle,
    Streaming(Mutex<Receiver<AgentMessageDelta>>),
}

impl Clone for AgentStatus {
    fn clone(&self) -> Self {
        Self::Idle
    }
}
