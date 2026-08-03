/// The wire format used to communicate with an LLM API.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ApiDialect {
    /// DeepSeek's OpenAI-compatible Chat Completions format.
    #[default]
    DeepSeekChatCompletions,
    /// DeepSeek's OpenAI-compatible Responses API format.
    DeepSeekResponses,
}
