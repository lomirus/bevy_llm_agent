use crate::{
    AgentMessage,
    agent::{Agent, AgentStatus},
};
use bevy::prelude::*;

pub(crate) fn write_agent_message(
    mut agents: Query<(Entity, &mut Agent)>,
    mut writer: MessageWriter<AgentMessage>,
) {
    for (entity, mut agent) in agents.iter_mut() {
        if let AgentStatus::Streaming(receiver) = &mut agent.status {
            let receiver = receiver.lock().unwrap();
            while let Ok(delta) = receiver.try_recv() {
                writer.write(AgentMessage { entity, delta });
            }
        }
    }
}
