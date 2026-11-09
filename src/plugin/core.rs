use crate::{
    messages::agent_message::AgentMessage,
    plugin::systems::{receive_responses, run_tool_calls},
};
use bevy::prelude::*;

pub struct LlmAgentPlugin;

/// The systems that stream agent responses and run their tools.
///
/// [`LlmAgentPlugin`] adds them to [`PreUpdate`], so this set only orders systems in that
/// schedule: `app.add_systems(PreUpdate, my_system.after(AgentSystems))`.
/// [`AgentMessage`]s written by them can be read in [`Update`] of the same frame.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AgentSystems;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AgentMessage>().add_systems(
            PreUpdate,
            (receive_responses, run_tool_calls)
                .chain()
                .in_set(AgentSystems),
        );
    }
}
