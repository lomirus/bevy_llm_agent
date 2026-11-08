#[derive(Clone)]
pub enum DialogMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: String,
        reasoning_content: String,
        tool_calls: Vec<ToolCall>,
        encrypted_reasoning: EncryptedReasoning,
    },
    Tool {
        id: String,
        result: String,
    },
}

impl DialogMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self::System {
            content: content.into(),
        }
    }
}

#[derive(Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Reasoning a provider returned encrypted, such as OpenAI's, which is sent back unchanged in
/// later requests so the model keeps its context across tool calls.
#[derive(Clone, Debug, Default)]
pub struct EncryptedReasoning(pub(crate) Vec<serde_json::Value>);
