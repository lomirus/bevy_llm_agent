mod tool;
mod tool_invocation;
mod tool_trait;

pub use tool::Tool;
pub use tool_invocation::ToolInvocation;
pub use tool_trait::ToolTrait;

use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = AgentTools)]
pub struct ToolOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ToolOf)]
pub struct AgentTools(Vec<Entity>);

#[derive(Message)]
pub(crate) struct RawToolInvocation {
    pub(crate) raw_args: String,
    pub(crate) raw_responder: oneshot::Sender<String>,
    pub(crate) dispatch:
        fn(&mut Commands, raw_args: String, raw_responder: oneshot::Sender<String>) -> (),
}
