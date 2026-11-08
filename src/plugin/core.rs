use crate::{
    messages::{agent_message::AgentMessage, user_message::UserMessage},
    plugin::systems::{read_user_message, receive_responses, run_tool_calls},
};
use bevy::prelude::*;

pub struct LlmAgentPlugin;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<UserMessage>()
            .add_message::<AgentMessage>()
            .add_systems(
                FixedUpdate,
                (read_user_message, receive_responses, run_tool_calls).chain(),
            );
    }
}
