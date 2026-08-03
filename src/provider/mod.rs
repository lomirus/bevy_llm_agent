pub mod deepseek;

/// Describes an LLM provider endpoint and model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Provider<'a> {
    pub base_url: &'a str,
    pub model: &'a str,
    pub api_key_env: &'a str,
}

impl<'a> Provider<'a> {
    pub const fn new(base_url: &'a str, model: &'a str, api_key_env: &'a str) -> Self {
        Self {
            base_url,
            model,
            api_key_env,
        }
    }
}
