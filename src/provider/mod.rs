//! Model presets for [`Agent::builder`](crate::agent::Agent::builder).

pub mod deepseek;
pub mod moonshot;

use crate::agent::{ApiDialect, Thinking};

/// Endpoint, credential source, and thinking defaults for a particular model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelPreset<'a> {
    pub api_dialect: ApiDialect,
    pub base_url: &'a str,
    pub model: &'a str,
    pub api_key_env: &'a str,
    pub supported_thinking: &'a [Thinking],
    pub default_thinking: Thinking,
}
