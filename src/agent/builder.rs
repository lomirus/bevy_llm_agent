use crate::{
    agent::{Agent, Thinking},
    provider::ModelPreset,
};
use std::{env, error::Error, fmt};

/// An agent configuration builder. See [`Agent::builder`].
#[derive(Clone)]
pub struct AgentBuilder<'a> {
    preset: ModelPreset<'a>,
    base_url: String,
    thinking: Thinking,
    api_key_source: ApiKeySource,
}

#[derive(Clone)]
enum ApiKeySource {
    Environment(String),
    Explicit(String),
}

impl<'a> AgentBuilder<'a> {
    pub(crate) fn new(preset: ModelPreset<'a>) -> Self {
        Self {
            preset,
            base_url: preset.base_url.to_owned(),
            thinking: preset.default_thinking,
            api_key_source: ApiKeySource::Environment(preset.api_key_env.to_owned()),
        }
    }

    /// Override the model's default thinking level.
    pub fn thinking(mut self, thinking: Thinking) -> Self {
        self.thinking = thinking;
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
        if !self.preset.supported_thinking.contains(&self.thinking) {
            return Err(AgentBuildError::UnsupportedThinking {
                model: self.preset.model.to_owned(),
                thinking: self.thinking,
            });
        }

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
            thinking: self.thinking,
            dialog: Vec::new(),
            api_base_url: self.base_url.trim_end_matches('/').to_owned(),
            api_key,
        })
    }
}

/// An agent configuration or credential error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AgentBuildError {
    UnsupportedThinking { model: String, thinking: Thinking },
    MissingApiKeyEnv { variable: String },
    NonUnicodeApiKeyEnv { variable: String },
}

impl fmt::Display for AgentBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedThinking { model, thinking } => {
                write!(f, "{thinking:?} thinking is not supported by model {model}")
            }
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
