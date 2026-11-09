mod tools;

use std::io::Write;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{prelude::*, provider::deepseek};
use tools::{AddToCounter, GetCounter};

#[derive(Resource)]
struct Counter(usize);

fn setup(mut commands: Commands) {
    commands
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
        .prompt(
            "By calling add_to_counter more than once, let the final value obtained by get_counter greater than 10.",
        );
}

fn print_text(
    mut agent_messages: MessageReader<AgentMessage>,
    mut app_exit: MessageWriter<AppExit>,
    counter: Res<Counter>,
    mut tool_call_open: Local<bool>,
) {
    for AgentMessage { delta, .. } in agent_messages.read() {
        use AgentMessageDelta::*;
        match delta {
            Content(text) => {
                print!("{text}");
                std::io::stdout().flush().unwrap();
            }
            ToolCallStart { id, name } => {
                if *tool_call_open {
                    println!(")");
                } else {
                    println!();
                }
                print!("[TOOL INVOKE] {id} => {name}(");
                *tool_call_open = true;
            }
            ToolCallArguments { arguments, .. } => print!("{arguments}"),
            ToolResult { id, content } => {
                if *tool_call_open {
                    println!(")");
                    *tool_call_open = false;
                }
                println!("[TOOL RESULT] {id} => {content}");
            }
            Finished(Ok(())) => {
                app_exit.write(if counter.0 > 10 {
                    AppExit::Success
                } else {
                    AppExit::error()
                });
            }
            Finished(Err(error)) => {
                error!("{error}");
                app_exit.write(AppExit::error());
            }
            ReasoningContent(_) => {}
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
        .add_systems(Startup, setup)
        .add_systems(Update, print_text)
        .run();
}
