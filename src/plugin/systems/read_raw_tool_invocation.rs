use crate::plugin::RawToolInvocationReceiver;
use bevy::prelude::*;

pub(crate) fn read_raw_tool_invocation(
    raw_tool_invocation_receiver: Res<RawToolInvocationReceiver>,
    mut commands: Commands,
) {
    for raw_tool_invocation in raw_tool_invocation_receiver.try_iter() {
        (raw_tool_invocation.dispatch)(
            &mut commands,
            raw_tool_invocation.raw_args,
            raw_tool_invocation.raw_responder,
        );
    }
}
