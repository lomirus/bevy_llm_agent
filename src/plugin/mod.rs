mod core;
mod systems;

use crate::tool::RawToolCall;
use bevy::prelude::*;

pub use core::LlmAgentPlugin;

#[derive(Resource, Deref)]
struct RawToolCallReceiver(crossbeam::channel::Receiver<RawToolCall>);

#[derive(Resource, Deref)]
struct RawToolCallSender(crossbeam::channel::Sender<RawToolCall>);
