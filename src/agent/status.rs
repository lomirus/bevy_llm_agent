use crate::messages::agent_message::AgentMessageDelta;
use bevy::prelude::*;
use std::sync::{Mutex, mpsc::Receiver};

#[derive(Default, Component)]
pub(crate) enum AgentStatus {
    #[default]
    Idle,
    Streaming(Mutex<Receiver<AgentMessageDelta>>),
}
