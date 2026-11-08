# bevy-llm-agent

A Bevy plugin for integrating LLM agents into Bevy applications.

> [!WARNING]
> **Work in progress:** This project is under active development, incomplete, and not ready for production use. APIs may change without notice.

## Compatibility

| Bevy | bevy-llm-agent |
|------|----------------|
| 0.20 | 0.1 (WIP)      |

## Basic Usage

### Create a Tool (Optional)

```rust
/// Get the weather for a city.
#[agent_tool(args = WeatherArgs, output = String, system = get_weather)]
#[derive(SceneComponent, JsonSchema, Clone, Default)]
struct GetWeather;

#[derive(Deserialize, JsonSchema)]
struct WeatherArgs {
    /// City whose weather should be queried.
    city: String,
}

fn get_weather(mut calls: MessageMutator<ToolCall<GetWeather>>) {
    for call in calls.read() {
        call.respond(format!("It is sunny in {}.", call.args.city)).unwrap();
    }
}
```

### Set Up the App

```rust
App::new()
    .add_agent_tool::<GetWeather>()
    .add_plugins((
        DefaultPlugins,
        LlmAgentPlugin,
    ))
    .run();
```

### Create an Agent

```bash
export DEEPSEEK_API_KEY="sk-xxx"
```

```rust
bsn! {
    Agent::builder(deepseek::V4_FLASH)
        .thinking(deepseek::FlashThinking::Off)
        .build()
        .unwrap()
    AgentTools [
        @GetWeather
    ]
}
```

### Send a Message

```rust
fn send_message(
    mut messages: MessageWriter<UserMessage>,
    agent: Single<Entity, With<Agent>>,
) {
    messages.write(UserMessage::new(
        *agent,
        "What is the weather in Shenzhen?",
    ));
}
```

### Handle Messages

```rust
fn handle_agent_messages(mut messages: MessageReader<AgentMessage>) {
    for message in messages.read() {
        if let Ok(AgentMessageDelta::Content(text)) = &message.delta {
            print!("{text}");
        }
    }
}
```

## License

Dual-licensed under either MIT or Apache-2.0.
