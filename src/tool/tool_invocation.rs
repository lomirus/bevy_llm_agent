use bevy::prelude::*;
use std::sync::Mutex;

use crate::tool::ToolTrait;

#[derive(Message)]
pub struct ToolInvocation<T: ToolTrait> {
    pub args: T::Args,
    pub(crate) responder: Mutex<Option<oneshot::Sender<String>>>,
}

impl<T: ToolTrait> ToolInvocation<T> {
    pub fn respond(&self, output: T::Output) {
        let sender = self.responder.lock().unwrap().take().unwrap();
        let output = serde_json::to_string(&output).unwrap();
        sender.send(output).unwrap();
    }
}
