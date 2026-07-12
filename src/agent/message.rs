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
    },
    Tool {
        id: String,
        result: String,
    },
}

#[derive(Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}
