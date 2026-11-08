use crate::{
    agent::{Agent, DialogMessage},
    provider::{ModelPreset, Thinking},
};
use std::{env, error::Error, fmt};

/// An agent configuration builder. See [`Agent::builder`].
#[derive(Clone)]
pub struct AgentBuilder<'a, T> {
    preset: ModelPreset<'a, T>,
    base_url: String,
    thinking: T,
    api_key_source: ApiKeySource,
    system_prompt: Option<String>,
}

#[derive(Clone)]
enum ApiKeySource {
    Environment(String),
    Explicit(String),
}

impl<'a, T: Thinking> AgentBuilder<'a, T> {
    pub(crate) fn new(preset: ModelPreset<'a, T>) -> Self {
        Self {
            preset,
            base_url: preset.base_url.to_owned(),
            thinking: T::default(),
            api_key_source: ApiKeySource::Environment(preset.api_key_env.to_owned()),
            system_prompt: None,
        }
    }

    /// Override the model's default thinking level.
    ///
    /// Only the options of the preset's own thinking type are accepted:
    ///
    /// ```compile_fail
    /// use bevy_llm_agent::{agent::Agent, provider::{deepseek, moonshot}};
    ///
    /// // `kimi-k3` takes `moonshot::K3Thinking`, which has no `Off`.
    /// Agent::builder(moonshot::K3).thinking(deepseek::Thinking::Off);
    /// ```
    pub fn thinking(mut self, thinking: T) -> Self {
        self.thinking = thinking;
        self
    }

    /// Start the dialog with a system prompt.
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the API base URL without an operation path.
    /// Trailing slashes are removed on build.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Use an explicit API key, replacing any previous credential source.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key_source = ApiKeySource::Explicit(api_key.into());
        self
    }

    /// Use this environment variable, replacing any previous credential source.
    pub fn api_key_env(mut self, variable: impl Into<String>) -> Self {
        self.api_key_source = ApiKeySource::Environment(variable.into());
        self
    }

    /// Validate the configuration, resolve the API key, and create the agent.
    pub fn build(self) -> Result<Agent, AgentBuildError> {
        let api_key = match self.api_key_source {
            ApiKeySource::Explicit(key) => key,
            ApiKeySource::Environment(variable) => {
                env::var(&variable).map_err(|error| match error {
                    env::VarError::NotPresent => AgentBuildError::MissingApiKeyEnv { variable },
                    env::VarError::NotUnicode(_) => {
                        AgentBuildError::NonUnicodeApiKeyEnv { variable }
                    }
                })?
            }
        };

        Ok(Agent {
            api_dialect: self.preset.api_dialect,
            model: self.preset.model.to_owned(),
            reasoning: self.thinking.reasoning(),
            dialog: self
                .system_prompt
                .into_iter()
                .map(DialogMessage::system)
                .collect(),
            api_base_url: self.base_url.trim_end_matches('/').to_owned(),
            api_key,
        })
    }
}

/// An agent configuration or credential error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AgentBuildError {
    MissingApiKeyEnv { variable: String },
    NonUnicodeApiKeyEnv { variable: String },
}

impl fmt::Display for AgentBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKeyEnv { variable } => {
                write!(f, "API key environment variable {variable} is not set")
            }
            Self::NonUnicodeApiKeyEnv { variable } => {
                write!(
                    f,
                    "API key environment variable {variable} is not valid Unicode"
                )
            }
        }
    }
}

impl Error for AgentBuildError {}
