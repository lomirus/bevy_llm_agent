use bevy::prelude::*;
use std::{sync::Arc, thread};

use crate::{
    agent::Agent,
    api::{self, StreamEvent},
    tool::Tool,
};

/// Present on an [`Agent`] while it is responding to a [`UserMessage`](crate::messages::user_message::UserMessage),
/// including while its tools run. It is removed once the response finishes or fails.
#[derive(Component)]
pub struct Responding {
    pub(crate) state: ResponseState,
    /// The agent's tools, captured when the response started.
    pub(crate) tools: Arc<[Tool]>,
}

pub(crate) enum ResponseState {
    Streaming(crossbeam_channel::Receiver<StreamEvent>),
    AwaitingTools,
}

impl Responding {
    /// Send the agent's dialog to the model on a worker thread.
    pub(crate) fn start(agent: &Agent, tools: Arc<[Tool]>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let agent = agent.clone();
        let worker_tools = tools.clone();
        thread::spawn(move || {
            let end = api::stream_response(&agent, &worker_tools, &tx);
            let _ = tx.send(StreamEvent::End(end));
        });
        Responding {
            state: ResponseState::Streaming(rx),
            tools,
        }
    }
}
