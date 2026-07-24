use crate::{
    agent::{self, Agent, AgentStatus},
    messages::agent_message::{AgentMessage, AgentMessageDelta},
};
use bevy::prelude::*;

pub(crate) fn write_agent_message(
    mut agents: Query<(Entity, &mut Agent, &mut AgentStatus)>,
    mut writer: MessageWriter<AgentMessage>,
) {
    for (entity, mut agent, mut agent_status) in agents.iter_mut() {
        let Agent { dialog, .. } = &mut *agent;
        let mut finished = false;
        {
            let AgentStatus::Streaming(receiver) = &*agent_status else {
                continue;
            };
            let receiver = receiver.lock().unwrap();
            while let Ok(delta) = receiver.try_recv() {
                match &delta {
                    AgentMessageDelta::Finish(_) => finished = true,
                    _ => agent::apply_delta(dialog, &delta),
                }
                writer.write(AgentMessage { entity, delta });
            }
        }
        if finished {
            *agent_status = AgentStatus::Idle;
        }
    }
}
