use bevy::prelude::*;
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use std::{any::type_name, sync::Mutex, thread};

#[derive(Component)]
#[relationship(relationship_target = AgentTools)]
pub struct ToolOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ToolOf)]
pub struct AgentTools(Vec<Entity>);

#[derive(Message)]
pub struct ToolInvocation<T: ToolTrait> {
    pub args: T::Args,
    responder: Mutex<Option<oneshot::Sender<T::Output>>>,
}

impl<T: ToolTrait> ToolInvocation<T> {
    pub fn respond(&self, output: T::Output) {
        if let Some(sender) = self.responder.lock().unwrap().take() {
            sender.send(output).unwrap();
        }
    }
}

#[derive(Clone, Component)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub(crate) dispatch: fn(
        &mut World,
        raw_args: String,
        raw_responder: Mutex<Option<oneshot::Sender<String>>>,
    ) -> (),
}

impl Default for Tool {
    // See issue: https://github.com/bevyengine/bevy/issues/24739
    fn default() -> Self {
        Tool {
            name: String::new(),
            description: String::new(),
            parameters: serde_json::Value::Null,
            dispatch: |_, _, _| {},
        }
    }
}

impl Tool {
    pub fn of<T: ToolTrait + JsonSchema>() -> Self {
        let full_name = type_name::<T>().to_owned();
        let schema = serde_json::json!(schemars::schema_for!(T));
        Tool {
            name: full_name.split("::").last().unwrap().to_owned(),
            description: schema.get("description").unwrap().to_string(),
            parameters: schema,
            dispatch:
                |world: &mut World,
                 raw_args: String,
                 raw_responder: Mutex<Option<oneshot::Sender<String>>>| {
                    let args: T::Args = serde_json::from_str(&raw_args).unwrap();
                    let raw_responder = raw_responder.lock().unwrap().take().unwrap();
                    let (tx, rx) = oneshot::channel::<T::Output>();
                    world.write_message(ToolInvocation::<T> {
                        args,
                        responder: Mutex::new(Some(tx)),
                    });
                    thread::spawn(move || {
                        let result = rx.recv().unwrap();
                        let raw_result = serde_json::to_string(&result).unwrap();
                        raw_responder.send(raw_result);
                    });
                },
        }
    }
}

pub trait ToolTrait: 'static {
    type Args: DeserializeOwned + Send + Sync;
    type Output: Serialize + Send;
}
