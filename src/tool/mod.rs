use bevy::prelude::*;
use schemars::JsonSchema;
use std::{any::type_name, sync::Mutex};

#[derive(Component)]
#[relationship(relationship_target = AgentTools)]
pub struct ToolOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ToolOf)]
pub struct AgentTools(Vec<Entity>);

#[derive(Message)]
pub struct ToolInvocation<I, O> {
    pub args: I,
    output_sender: Mutex<Option<oneshot::Sender<O>>>,
}

impl<I, O> ToolInvocation<I, O> {
    pub fn send_back(&self, output: O) {
        if let Some(sender) = self.output_sender.lock().unwrap().take() {
            sender.send(output).unwrap();
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct Tool {
    pub name: String,
    pub desc: String,
    pub args: serde_json::Value,
}

impl Tool {
    pub fn from<T: JsonSchema>() -> Self {
        let full_name = type_name::<T>().to_owned();
        let schema = serde_json::json!(schemars::schema_for!(T));
        Tool {
            name: full_name.split("::").last().unwrap().to_owned(),
            desc: schema.get("description").unwrap().to_string(),
            args: schema,
        }
    }
}
