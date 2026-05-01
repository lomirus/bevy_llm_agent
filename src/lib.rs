pub mod agent;
pub mod prelude;
pub mod tool;

pub use rig::agent::MultiTurnStreamItem;
pub use rig::message::Message;
pub use rig::message::ToolResultContent;
pub use rig::providers::deepseek::Client;
pub use rig::streaming::StreamedAssistantContent;
pub use rig::streaming::StreamedUserContent;

use agent::{Agent, AgentStatus};
use bevy::prelude::*;
use rig::providers::deepseek::StreamingCompletionResponse;

#[derive(Message)]
pub struct StreamMessage {
    pub entity: Entity,
    pub delta: MultiTurnStreamItem<StreamingCompletionResponse>,
}

pub struct LlmPlugin;

impl Plugin for LlmPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<StreamMessage>()
            .add_systems(FixedUpdate, read_stream);
    }
}

fn read_stream(mut agents: Query<(Entity, &mut Agent)>, mut events: MessageWriter<StreamMessage>) {
    for (entity, mut agent) in agents.iter_mut() {
        if let AgentStatus::Streaming(receiver) = &mut agent.status {
            while let Ok(delta) = receiver.try_recv() {
                events.write(StreamMessage { entity, delta });
            }
        }
    }
}
