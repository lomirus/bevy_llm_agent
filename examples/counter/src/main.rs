mod tools;

use std::io::Write;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::{
    AssistantContent, Client, LlmPlugin, MultiTurnItem, ToolResultContent, UserContent,
    agent::Agent, prelude::*, tool::ToolAdapter,
};
use tools::{AddToCounter, GetCounter};

#[derive(Resource)]
struct Counter(usize);

fn setup(
    mut commands: Commands,
    add_to_counter: Res<ToolAdapter<AddToCounter>>,
    get_counter: Res<ToolAdapter<GetCounter>>,
) {
    let client = Client::from_env().unwrap();
    let agent = client
        .agent(DEEPSEEK_V4_FLASH)
        .default_max_turns(usize::MAX - 1)
        .tool(add_to_counter.clone())
        .tool(get_counter.clone())
        .build();
    let mut agent = Agent::new(agent);
    agent.streaming_chat(
        "By calling add_to_counter, the final value obtained by get_counter is greater than 10.",
    );
    commands.spawn(agent);
}

fn print_text(
    mut stream_messages: MessageReader<bevy_llm::StreamMessage>,
    mut app_exit: MessageWriter<AppExit>,
    counter: Res<Counter>,
) {
    for stream_message in stream_messages.read() {
        match &stream_message.delta {
            MultiTurnItem::StreamAssistantItem(message) => match message {
                AssistantContent::Text(text) => {
                    print!("{text}");
                    std::io::stdout().flush().unwrap();
                }
                AssistantContent::ToolCall { tool_call, .. } => {
                    println!();
                    info!(
                        "[TOOL CALL] {}({})",
                        tool_call.function.name, tool_call.function.arguments
                    );
                }
                _ => {}
            },
            MultiTurnItem::StreamUserItem(UserContent::ToolResult { tool_result, .. }) => {
                let text = match tool_result.content.first() {
                    ToolResultContent::Text(text) => text.text,
                    ToolResultContent::Image(_) => "<image>".to_string(),
                };
                info!("[TOOL RESULT] {:?}", text);
            }
            MultiTurnItem::FinalResponse(..) => {
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
            LlmPlugin,
        ))
        .register_llm_tool::<AddToCounter>()
        .register_llm_tool::<GetCounter>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, print_text)
        .run();
}
