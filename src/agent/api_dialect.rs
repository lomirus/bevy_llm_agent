/// The wire format used to communicate with an LLM API.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ApiDialect {
    /// The OpenAI-compatible Chat Completions format (`/chat/completions`).
    #[default]
    ChatCompletions,
    /// The OpenAI-compatible Responses format (`/responses`).
    Responses,
}
