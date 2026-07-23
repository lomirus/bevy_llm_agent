use bevy::prelude::*;
use schemars::JsonSchema;
use std::{any::type_name, sync::Mutex};

use crate::tool::{ToolInvocation, ToolTrait};

#[derive(Clone, Component)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub(crate) dispatch:
        fn(&mut Commands, raw_args: String, raw_responder: oneshot::Sender<String>) -> (),
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
        Tool {
            name: full_name.split("::").last().unwrap().to_owned(),
            description: serde_json::json!(schemars::schema_for!(T))
                .get("description")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            parameters: serde_json::json!(schemars::schema_for!(T::Args)),
            dispatch: |commands: &mut Commands,
                       raw_args: String,
                       raw_responder: oneshot::Sender<String>| {
                let args: T::Args = serde_json::from_str(&raw_args).unwrap();
                let raw_responder = raw_responder;
                commands.write_message(ToolInvocation::<T> {
                    args,
                    responder: Mutex::new(Some(raw_responder)),
                });
            },
        }
    }
}
