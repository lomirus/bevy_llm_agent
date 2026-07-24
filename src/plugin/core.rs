use crate::{
    messages::{agent_message::AgentMessage, user_message::UserMessage},
    plugin::{
        RawToolInvocationReceiver, RawToolInvocationSender,
        systems::{read_raw_tool_invocation, read_user_message, write_agent_message},
    },
};
use bevy::prelude::*;

pub struct LlmAgentPlugin;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let (tx, rx) = crossbeam::channel::unbounded();
        app.add_message::<UserMessage>()
            .add_message::<AgentMessage>()
            .insert_resource(RawToolInvocationSender(tx))
            .insert_resource(RawToolInvocationReceiver(rx))
            .add_systems(
                FixedUpdate,
                (
                    read_user_message,
                    write_agent_message,
                    read_raw_tool_invocation,
                ),
            );
    }
}
