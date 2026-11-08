use crate::{plugin::RawToolCallReceiver, tool::RawToolCall};
use bevy::prelude::*;

pub(crate) fn run_tool_calls(world: &mut World) {
    let calls: Vec<RawToolCall> = world.resource::<RawToolCallReceiver>().try_iter().collect();
    for RawToolCall {
        entity,
        raw_args,
        raw_responder,
        call,
    } in calls
    {
        let result = call(world, entity, &raw_args);
        // The worker only stops waiting if it has already exited.
        let _ = raw_responder.send(result);
    }
}
