pub mod deepseek;

use crate::agent::{ApiDialect, Thinking};

/// Describes an LLM provider endpoint, model, and API dialect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Provider<'a> {
    pub api_dialect: ApiDialect,
    pub base_url: &'a str,
    pub model: &'a str,
    pub api_key_env: &'a str,
    pub thinking: Thinking,
}

/// Builds a provider after validating its thinking configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderBuilder<'a> {
    pub api_dialect: ApiDialect,
    pub base_url: &'a str,
    pub model: &'a str,
    pub api_key_env: &'a str,
    pub supported_thinking: &'a [Thinking],
}

impl<'a> ProviderBuilder<'a> {
    pub fn build(self, thinking: Thinking) -> Result<Provider<'a>, String> {
        if !self.supported_thinking.contains(&thinking) {
            return Err(format!(
                "{thinking:?} thinking is not supported by model {}",
                self.model
            ));
        }

        Ok(Provider {
            api_dialect: self.api_dialect,
            base_url: self.base_url,
            model: self.model,
            api_key_env: self.api_key_env,
            thinking,
        })
    }
}
