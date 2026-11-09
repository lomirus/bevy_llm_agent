//! Model presets for [`Agent::builder`](crate::agent::Agent::builder).

pub mod deepseek;
pub mod moonshot;
pub mod openai;
mod thinking;

use std::marker::PhantomData;

use crate::agent::ApiDialect;

pub use thinking::Thinking;
pub(crate) use thinking::sealed::{Reasoning, ReasoningEffort, ReasoningFormat, Sealed};

/// Endpoint and credential source for a particular model.
///
/// To pick values at runtime, e.g. from a config file, start from a preset and override them
/// with [`AgentBuilder`](crate::agent::AgentBuilder).
///
/// `T` is the set of thinking options the model accepts; its
/// [`Default`] value is the model's default thinking level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelPreset<T> {
    pub api_dialect: ApiDialect,
    pub base_url: &'static str,
    pub model: &'static str,
    pub api_key_env: &'static str,
    thinking: PhantomData<T>,
}

impl<T: Thinking> ModelPreset<T> {
    /// Describe a model that accepts the thinking options `T`.
    ///
    /// ```
    /// use bevy_llm_agent::{agent::ApiDialect, provider::{ModelPreset, deepseek}};
    ///
    /// let preset = ModelPreset::<deepseek::Thinking>::new(
    ///     ApiDialect::ChatCompletions,
    ///     "https://api.deepseek.com",
    ///     "deepseek-v4-pro",
    ///     "DEEPSEEK_API_KEY",
    /// );
    /// ```
    pub const fn new(
        api_dialect: ApiDialect,
        base_url: &'static str,
        model: &'static str,
        api_key_env: &'static str,
    ) -> Self {
        Self {
            api_dialect,
            base_url,
            model,
            api_key_env,
            thinking: PhantomData,
        }
    }
}
