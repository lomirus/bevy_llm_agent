use crate::tool::{ToolInvocation, ToolTrait};
use bevy::prelude::*;

pub trait AppExt {
    fn add_agent_tool<T: ToolTrait>(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn add_agent_tool<T: ToolTrait>(&mut self) -> &mut Self {
        self.add_message::<ToolInvocation<T>>()
            .add_systems(FixedUpdate, T::boxed_system())
    }
}
