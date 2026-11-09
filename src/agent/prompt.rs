use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    agent::{Agent, DialogMessage, Responding},
    tool::{AgentTools, Tool},
};

/// Prompts an [`Agent`] through [`EntityCommands`].
pub trait AgentCommandsExt {
    /// Add a user message to the agent's dialog and start responding to it.
    ///
    /// [`Responding`] is inserted right away, and the response streams in as
    /// [`AgentMessage`](crate::messages::agent_message::AgentMessage)s.
    /// The prompt is ignored with a warning if the agent is already responding.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_llm_agent::prelude::*;
    /// fn ask(mut commands: Commands, agent: Single<Entity, With<Agent>>) {
    ///     commands.entity(*agent).prompt("What is the weather in Shenzhen?");
    /// }
    /// ```
    fn prompt(&mut self, prompt: impl Into<String>) -> &mut Self;
}

impl AgentCommandsExt for EntityCommands<'_> {
    fn prompt(&mut self, prompt: impl Into<String>) -> &mut Self {
        let prompt = prompt.into();
        self.queue(move |entity: EntityWorldMut| start_response(entity, prompt))
    }
}

fn start_response(mut entity: EntityWorldMut, prompt: String) {
    let id = entity.id();
    if entity.contains::<Responding>() {
        warn!("ignoring prompt; agent {id} is already responding");
        return;
    }
    let tools: Arc<[Tool]> = entity
        .get::<AgentTools>()
        .into_iter()
        .flat_map(AgentTools::iter)
        .filter_map(|tool| entity.world().get::<Tool>(tool))
        .cloned()
        .collect();
    let Some(mut agent) = entity.get_mut::<Agent>() else {
        warn!("ignoring prompt; {id} is not an agent");
        return;
    };
    agent.dialog.push(DialogMessage::User { content: prompt });
    let responding = Responding::start(&agent, tools);
    entity.insert(responding);
}
