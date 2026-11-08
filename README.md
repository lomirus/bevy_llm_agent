# bevy-llm-agent

A Bevy plugin for integrating LLM agents into Bevy applications.

> [!WARNING]
> **Work in progress:** This project is under active development, incomplete, and not ready for production use. APIs may change without notice.

## Compatibility

| Bevy | bevy-llm-agent |
|------|----------------|
| 0.20 | 0.1 (WIP)      |

## Basic Usage

The types used below are available through `use bevy_llm_agent::prelude::*;`.

### Create a Tool (Optional)

```rust
/// Get the weather for a city.
#[agent_tool]
struct GetWeather;

#[derive(Deserialize, JsonSchema)]
struct WeatherArgs {
    /// City whose weather should be queried.
    city: String,
}

fn get_weather(In(input): In<ToolInput<WeatherArgs>>) -> String {
    format!("It is sunny in {}.", input.args.city)
}
```

The tool is named after the struct in snake case (`get_weather`) and runs the system of the same name; use `#[agent_tool(other_system)]` to pick another one. Its doc comment is sent to the model as the tool description, and the system's return value is serialized to JSON and sent back to the model.

### Set Up the App

```rust
App::new()
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
    Agent::builder(deepseek::FLASH)
        .thinking(deepseek::Thinking::Off)
        .build()
        .unwrap()
    AgentTools [
        @GetWeather
    ]
}
```

Presets are available for DeepSeek (`provider::deepseek`), Moonshot (`provider::moonshot`), and OpenAI (`provider::openai`). Other OpenAI-compatible services can be described with `ModelPreset::new`.

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
