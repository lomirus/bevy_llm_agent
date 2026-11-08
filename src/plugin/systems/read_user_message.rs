use crate::{
    agent::{Agent, DialogMessage, Responding},
    messages::user_message::UserMessage,
    tool::{AgentTools, Tool},
};
use bevy::{ecs::entity::EntityHashSet, prelude::*};

pub(crate) fn read_user_message(
    mut reader: MessageReader<UserMessage>,
    mut agents: Query<(&mut Agent, Has<Responding>, Option<&AgentTools>)>,
    tools: Query<&Tool>,
    mut commands: Commands,
) {
    // `Responding` is inserted through commands, so track agents started in this run too.
    let mut started = EntityHashSet::default();
    for UserMessage { entity, prompt } in reader.read() {
        let Ok((mut agent, responding, agent_tools)) = agents.get_mut(*entity) else {
            warn!("ignoring user message; {entity} is not an agent");
            continue;
        };
        if responding || !started.insert(*entity) {
            warn!("ignoring user message; agent {entity} is already responding");
            continue;
        }

        agent.dialog.push(DialogMessage::User {
            content: prompt.clone(),
        });
        let tools = tools
            .iter_many(agent_tools.into_iter().flat_map(AgentTools::iter))
            .flatten()
            .cloned()
            .collect();
        commands
            .entity(*entity)
            .insert(Responding::start(&agent, tools));
    }
}
