mod core;
mod systems;

use crate::tool::RawToolInvocation;
use bevy::prelude::*;

pub use core::LlmAgentPlugin;

#[derive(Resource, Deref)]
struct RawToolInvocationReceiver(crossbeam::channel::Receiver<RawToolInvocation>);

#[derive(Resource, Deref)]
struct RawToolInvocationSender(crossbeam::channel::Sender<RawToolInvocation>);
