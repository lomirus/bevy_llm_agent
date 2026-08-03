pub mod deepseek;

use crate::agent::ApiDialect;

/// Describes an LLM provider endpoint, model, and API dialect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Provider<'a> {
    pub api_dialect: ApiDialect,
    pub base_url: &'a str,
    pub model: &'a str,
    pub api_key_env: &'a str,
}

impl<'a> Provider<'a> {
    pub const fn new(
        api_dialect: ApiDialect,
        base_url: &'a str,
        model: &'a str,
        api_key_env: &'a str,
    ) -> Self {
        Self {
            api_dialect,
            base_url,
            model,
            api_key_env,
        }
    }
}
