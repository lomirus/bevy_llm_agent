mod tools;

use std::io::Write;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AppExt, LlmAgentPlugin,
    agent::Agent,
    messages::{
        agent_message::{AgentMessage, AgentMessageDelta, AgentMessageError},
        user_message::UserMessage,
    },
    provider::deepseek,
    tool::AgentTools,
};
use tools::{AddToCounter, GetCounter};

#[derive(Resource)]
struct Counter(usize);

fn setup(mut commands: Commands, mut sender: MessageWriter<UserMessage>) {
    let entity = commands
        .spawn_scene(bsn! {
                Agent::builder(deepseek::FLASH)
                    .thinking(deepseek::Thinking::Off)
                    .build()
                    .unwrap()
                AgentTools [
                    @GetCounter
                    --
                    @AddToCounter
                ]
        })
        .id();

    sender.write(UserMessage::new(
            entity,
            "By calling add_to_counter more than once, let the final value obtained by get_counter greater than 10.",
    ));
}

fn print_text(
    mut agent_messages: MessageReader<AgentMessage>,
    mut app_exit: MessageWriter<AppExit>,
    counter: Res<Counter>,
    mut tool_call_open: Local<bool>,
) {
    for agent_message in agent_messages.read() {
        use AgentMessageDelta::*;
        let delta = match &agent_message.delta {
            Ok(delta) => delta,
            Err(AgentMessageError::Other(msg)) => {
                error!("{msg}");
                app_exit.write(AppExit::error());
                continue;
            }
            Err(err) => {
                error!("{err:?}");
                app_exit.write(AppExit::error());
                continue;
            }
        };
        match delta {
            Content(text) => {
                print!("{text}");
                std::io::stdout().flush().unwrap();
            }
            ToolCall {
                name,
                arguments,
                tool_call_id,
            } => {
                if !name.is_empty() {
                    if *tool_call_open {
                        println!(")");
                    } else {
                        println!();
                    }
                    print!("[TOOL INVOKE] {tool_call_id} => {name}(");
                    *tool_call_open = true;
                }
                print!("{arguments}");
            }
            ToolResult {
                content,
                tool_call_id,
            } => {
                if *tool_call_open {
                    println!(")");
                    *tool_call_open = false;
                }
                println!("[TOOL RESULT] {} => {}", tool_call_id, content);
            }
            Finish => {
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
                filter: DEFAULT_FILTER.to_owned(),
                ..default()
            }),
            LlmAgentPlugin,
        ))
        .add_agent_tool::<GetCounter>()
        .add_agent_tool::<AddToCounter>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, print_text)
        .run();
}
