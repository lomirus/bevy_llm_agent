use crate::{
    agent::{self, Agent, DialogMessage, Responding, ResponseState},
    messages::agent_message::{AgentMessage, AgentMessageDelta, AgentMessageError},
};
use bevy::prelude::*;

/// Runs the tool calls of agents whose stream ended with them, then continues their responses.
pub(crate) fn run_tool_calls(
    world: &mut World,
    agents: &mut QueryState<(Entity, &Agent, &Responding)>,
) {
    let pending: Vec<_> = agents
        .iter(world)
        .filter(|(_, _, responding)| matches!(responding.state, ResponseState::AwaitingTools))
        .map(|(entity, agent, responding)| {
            let tool_calls = match agent.dialog.last() {
                Some(DialogMessage::Assistant { tool_calls, .. }) => tool_calls.clone(),
                _ => Vec::new(),
            };
            (entity, tool_calls, responding.tools.clone())
        })
        .collect();

    'agents: for (entity, tool_calls, tools) in pending {
        if tool_calls.is_empty() {
            world.write_message(AgentMessage {
                entity,
                delta: Err(AgentMessageError::Other(
                    "the model stopped for tool calls without making any".to_owned(),
                )),
            });
            world.entity_mut(entity).remove::<Responding>();
            continue;
        }

        for tool_call in tool_calls {
            let content = match tools.iter().find(|tool| tool.name == tool_call.name) {
                Some(tool) => (tool.call)(world, entity, &tool_call.arguments),
                None => format!("Error: unknown tool `{}`", tool_call.name),
            };
            let delta = AgentMessageDelta::ToolResult {
                content,
                tool_call_id: tool_call.id,
            };
            // A tool may have despawned the agent.
            let Some(mut agent) = world.get_mut::<Agent>(entity) else {
                continue 'agents;
            };
            agent::apply_delta(&mut agent.dialog, &delta);
            world.write_message(AgentMessage {
                entity,
                delta: Ok(delta),
            });
        }

        let responding = Responding::start(world.get::<Agent>(entity).unwrap(), tools);
        world.entity_mut(entity).insert(responding);
    }
}
