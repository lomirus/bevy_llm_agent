use crate::agent::ApiDialect;

#[derive(Clone, Debug)]
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
        raw_output: RawOutput,
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

#[derive(Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Output items a provider needs back unchanged in later requests so the model keeps its
/// reasoning across tool calls: OpenAI's encrypted reasoning items, or the whole turn for
/// Anthropic and Gemini, whose signed thinking has to stay in place among the text and tool
/// calls it was produced with.
///
/// The items are only sent back through the [`ApiDialect`] that produced them. When they make up
/// the whole turn, they are sent instead of the message's `content` and `tool_calls`.
#[derive(Clone, Debug, Default)]
pub struct RawOutput {
    dialect: Option<ApiDialect>,
    items: Vec<serde_json::Value>,
}

impl RawOutput {
    /// The items produced through `dialect`.
    pub(crate) fn items(&self, dialect: ApiDialect) -> &[serde_json::Value] {
        if self.dialect == Some(dialect) {
            &self.items
        } else {
            &[]
        }
    }

    /// Add an item, dropping any produced through another dialect.
    pub(crate) fn push(&mut self, dialect: ApiDialect, item: serde_json::Value) {
        if self.dialect != Some(dialect) {
            self.dialect = Some(dialect);
            self.items.clear();
        }
        self.items.push(item);
    }
}
