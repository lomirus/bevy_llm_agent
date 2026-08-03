use crate::{
    messages::{agent_message::AgentMessage, user_message::UserMessage},
    plugin::{
        RawToolCallReceiver, RawToolCallSender,
        systems::{read_raw_tool_invocation, read_user_message, write_agent_message},
    },
};
use bevy::prelude::*;

pub struct LlmAgentPlugin;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let (tx, rx) = crossbeam_channel::unbounded();
        app.add_message::<UserMessage>()
            .add_message::<AgentMessage>()
            .insert_resource(RawToolCallSender(tx))
            .insert_resource(RawToolCallReceiver(rx))
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
