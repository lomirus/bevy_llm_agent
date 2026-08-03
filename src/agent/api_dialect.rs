/// The wire format used to communicate with an LLM API.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ApiDialect {
    #[default]
    ChatCompletion,
    /// Reserved for future support.
    ResponsesApi,
}
