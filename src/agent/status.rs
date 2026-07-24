use std::sync::{Mutex, mpsc::Receiver};
use bevy::prelude::*;
use crate::messages::agent_message::AgentMessageDelta;

#[derive(Default, Component)]
pub(crate) enum AgentStatus {
    #[default]
    Idle,
    Streaming(Mutex<Receiver<AgentMessageDelta>>),
}
