mod core;
mod systems;

use crate::tool::RawToolCall;
use bevy::prelude::*;

pub use core::LlmAgentPlugin;

#[derive(Resource, Deref)]
struct RawToolCallReceiver(crossbeam_channel::Receiver<RawToolCall>);

#[derive(Resource, Deref)]
struct RawToolCallSender(crossbeam_channel::Sender<RawToolCall>);
