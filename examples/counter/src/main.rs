mod tools;

use std::io::Write;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AgentMessageDelta, DEEPSEEK_V4_FLASH, LlmAgentPlugin, UserMessage,
    agent::{Agent, Thinking},
    tool::{AgentTools, Tool, ToolInvocation},
};
use tools::{AddToCounter, GetCounter};

use crate::tools::{add_to_counter, get_counter};

#[derive(Resource)]
struct Counter(usize);

fn setup(mut commands: Commands, mut sender: MessageWriter<UserMessage>) {
    let get_counter_tool = Tool::of::<GetCounter>();
    let add_to_counter_tool = Tool::of::<AddToCounter>();
    let entity = commands
        .spawn_scene(bsn! {
            Agent::new(DEEPSEEK_V4_FLASH, Thinking::Off)
            AgentTools [
                template_value(get_counter_tool),
                template_value(add_to_counter_tool)
            ]
        })
        .id();

    sender.write(UserMessage::new(
        entity,
        "By calling add_to_counter, the final value obtained by get_counter is greater than 10.",
    ));
}

fn print_text(
    mut agent_messages: MessageReader<bevy_llm_agent::AgentMessage>,
    mut app_exit: MessageWriter<AppExit>,
    counter: Res<Counter>,
) {
    for agent_message in agent_messages.read() {
        use AgentMessageDelta::*;
        match &agent_message.delta {
            Content(text) => {
                print!("{text}");
                std::io::stdout().flush().unwrap();
            }
            ToolCall {
                name, arguments, ..
            } => {
                println!();
                info!("[TOOL CALL] {}({})", name, arguments);
            }
            ToolResult { content, .. } => {
                info!("[TOOL RESULT] {}", content);
            }
            Finish(_) => {
                app_exit.write(if counter.0 > 10 {
                    AppExit::Success
                } else {
                    AppExit::error()
                });
            }
            _ => {}
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Counter(0))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},rig::agent::prompt_request::streaming=warn"),
                ..default()
            }),
            LlmAgentPlugin,
        ))
        .add_message::<ToolInvocation<GetCounter>>()
        .add_message::<ToolInvocation<AddToCounter>>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (print_text, get_counter, add_to_counter))
        .run();
}
