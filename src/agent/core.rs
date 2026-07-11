use bevy::prelude::*;
use rig::providers::deepseek::CompletionModel;

use super::AgentStatus;

#[derive(Component)]
pub struct Agent {
    pub(crate) agent: rig::agent::Agent<CompletionModel>,
    pub dialog: Vec<rig::message::Message>,
    pub(crate) status: AgentStatus,
}
