pub mod agent;
pub mod prelude;
pub mod tool;

pub use rig::agent::MultiTurnStreamItem as MultiTurnItem;
pub use rig::message::Message;
pub use rig::message::ToolResultContent;
pub use rig::streaming::StreamedAssistantContent as AssistantContent;
pub use rig::streaming::StreamedUserContent as UserContent;

use agent::{Agent, AgentStatus};
use bevy::prelude::*;
use rig::providers::deepseek::StreamingCompletionResponse as CompletionResponse;

#[derive(Message)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: MultiTurnItem<CompletionResponse>,
}

pub struct LlmAgentPlugin;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<AgentMessage>()
            .add_systems(FixedUpdate, read_agent_message);
    }
}

fn read_agent_message(mut agents: Query<(Entity, &mut Agent)>, mut events: MessageWriter<AgentMessage>) {
    for (entity, mut agent) in agents.iter_mut() {
        if let AgentStatus::Streaming(receiver) = &mut agent.status {
            while let Ok(delta) = receiver.try_recv() {
                events.write(AgentMessage { entity, delta });
            }
        }
    }
}
