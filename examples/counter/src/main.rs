mod tools;

use std::io::Write;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AgentMessageDelta, AppExt, DEEPSEEK_V4_FLASH, LlmAgentPlugin, UserMessage,
    agent::{Agent, Thinking},
    tool::{AgentTools, Tool},
};
use tools::{AddToCounter, GetCounter};

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
        "By calling add_to_counter more than once, let the final value obtained by get_counter greater than 10.",
    ));
}

#[derive(Resource)]
enum OutputPhase {
    ToolCallName,
    ToolCallArgs,
    Others,
}

fn print_text(
    mut agent_messages: MessageReader<bevy_llm_agent::AgentMessage>,
    mut app_exit: MessageWriter<AppExit>,
    counter: Res<Counter>,
    mut output_phase: ResMut<OutputPhase>,
) {
    for agent_message in agent_messages.read() {
        use AgentMessageDelta::*;
        match &agent_message.delta {
            Content(text) => {
                print!("{text}");
                std::io::stdout().flush().unwrap();
            }
            ToolCall {
                name,
                arguments,
                tool_call_id,
            } => match *output_phase {
                OutputPhase::ToolCallName => {
                    assert_eq!(tool_call_id, "");
                    *output_phase = OutputPhase::ToolCallArgs;
                    print!("{arguments}");
                }
                OutputPhase::ToolCallArgs => {
                    if tool_call_id != "" {
                        *output_phase = OutputPhase::ToolCallName;
                        println!(")");
                        print!("[TOOL INVOKE] {tool_call_id} => {name}(");
                    } else {
                        print!("{arguments}");
                    }
                }
                OutputPhase::Others => {
                    assert_ne!(tool_call_id, "");
                    *output_phase = OutputPhase::ToolCallName;
                    println!();
                    print!("[TOOL INVOKE] {tool_call_id} => {name}(");
                }
            },
            ToolResult {
                content,
                tool_call_id,
            } => {
                if let OutputPhase::ToolCallArgs = *output_phase {
                    println!(")");
                    *output_phase = OutputPhase::Others;
                }
                println!("[TOOL RESULT] {} => {}", tool_call_id, content);
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
                filter: DEFAULT_FILTER.to_owned(),
                ..default()
            }),
            LlmAgentPlugin,
        ))
        .add_agent_tool::<GetCounter>()
        .add_agent_tool::<AddToCounter>()
        .insert_resource(OutputPhase::Others)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, print_text)
        .run();
}
