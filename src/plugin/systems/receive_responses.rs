use crate::{
    agent::{self, Agent, Responding, ResponseState},
    api::{StreamEvent, StreamOutcome},
    messages::agent_message::{AgentMessage, AgentMessageDelta, AgentMessageError},
};
use bevy::prelude::*;
use crossbeam_channel::TryRecvError;

pub(crate) fn receive_responses(
    mut agents: Query<(Entity, &mut Agent, &mut Responding)>,
    mut writer: MessageWriter<AgentMessage>,
    mut commands: Commands,
) {
    for (entity, mut agent, mut responding) in &mut agents {
        let ResponseState::Streaming(receiver) = &responding.state else {
            continue;
        };
        let end = loop {
            match receiver.try_recv() {
                Ok(StreamEvent::Delta(delta)) => {
                    agent::apply_delta(&mut agent.dialog, &delta);
                    writer.write(AgentMessage {
                        entity,
                        delta: Ok(delta),
                    });
                }
                Ok(StreamEvent::End(end)) => break Some(end),
                Err(TryRecvError::Empty) => break None,
                Err(TryRecvError::Disconnected) => {
                    break Some(Err(AgentMessageError::Other(
                        "agent worker exited unexpectedly".to_owned(),
                    )));
                }
            }
        };
        let delta = match end {
            None => continue,
            Some(Ok(StreamOutcome::ToolCalls)) => {
                responding.state = ResponseState::AwaitingTools;
                continue;
            }
            Some(Ok(StreamOutcome::Finished)) => Ok(AgentMessageDelta::Finish),
            Some(Err(error)) => Err(error),
        };
        writer.write(AgentMessage { entity, delta });
        commands.entity(entity).remove::<Responding>();
    }
}
