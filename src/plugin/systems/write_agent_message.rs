use crate::{
    agent::{self, Agent, AgentStatus},
    messages::agent_message::{AgentMessage, AgentMessageDelta, AgentMessageError},
};
use bevy::prelude::*;
use std::sync::mpsc::TryRecvError;

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
            loop {
                match receiver.try_recv() {
                    Ok(delta) => {
                        match &delta {
                            Ok(AgentMessageDelta::Finish) | Err(_) => finished = true,
                            Ok(delta) => agent::apply_delta(dialog, delta),
                        }
                        writer.write(AgentMessage { entity, delta });
                        if finished {
                            break;
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        writer.write(AgentMessage {
                            entity,
                            delta: Err(AgentMessageError::Other(
                                "agent worker exited unexpectedly".to_owned(),
                            )),
                        });
                        finished = true;
                        break;
                    }
                }
            }
        }
        if finished {
            *agent_status = AgentStatus::Idle;
        }
    }
}
