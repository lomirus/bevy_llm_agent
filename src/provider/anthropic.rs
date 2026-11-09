//! Anthropic's Claude models, served through the Messages API.
//!
//! Reasoning streams as summaries, since Claude models hide it by default.

use super::{ModelPreset, Reasoning, ReasoningEffort, Sealed, Thinking};
use crate::agent::ApiDialect;

const BASE_URL: &str = "https://api.anthropic.com/v1";
const API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Thinking options for [`FABLE_5_1`], which always thinks.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FableThinking {
    Low,
    Medium,
    #[default]
    High,
    XHigh,
    Max,
}

impl Thinking for FableThinking {}

impl Sealed for FableThinking {
    fn reasoning(self) -> Reasoning {
        Reasoning::Enabled(Some(match self {
            Self::Low => ReasoningEffort::Low,
            Self::Medium => ReasoningEffort::Medium,
            Self::High => ReasoningEffort::High,
            Self::XHigh => ReasoningEffort::XHigh,
            Self::Max => ReasoningEffort::Max,
        }))
    }
}

/// Thinking options for [`OPUS_5_5`], which always thinks.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum OpusThinking {
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
}

impl Thinking for OpusThinking {}

impl Sealed for OpusThinking {
    fn reasoning(self) -> Reasoning {
        Reasoning::Enabled(Some(match self {
            Self::Low => ReasoningEffort::Low,
            Self::Medium => ReasoningEffort::Medium,
            Self::High => ReasoningEffort::High,
            Self::XHigh => ReasoningEffort::XHigh,
            Self::Max => ReasoningEffort::Max,
        }))
    }
}

/// Thinking options for [`SONNET_5_5`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SonnetThinking {
    /// No extended thinking; only short progress updates between tool calls are streamed as
    /// reasoning.
    Off,
    Low,
    Medium,
    #[default]
    High,
    XHigh,
    Max,
}

impl Thinking for SonnetThinking {}

impl Sealed for SonnetThinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::BetweenTools,
            Self::Low => Reasoning::Enabled(Some(ReasoningEffort::Low)),
            Self::Medium => Reasoning::Enabled(Some(ReasoningEffort::Medium)),
            Self::High => Reasoning::Enabled(Some(ReasoningEffort::High)),
            Self::XHigh => Reasoning::Enabled(Some(ReasoningEffort::XHigh)),
            Self::Max => Reasoning::Enabled(Some(ReasoningEffort::Max)),
        }
    }
}

/// Thinking options for [`HAIKU_5_5`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum HaikuThinking {
    Off,
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
}

impl Thinking for HaikuThinking {}

impl Sealed for HaikuThinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::Disabled,
            Self::Low => Reasoning::Enabled(Some(ReasoningEffort::Low)),
            Self::Medium => Reasoning::Enabled(Some(ReasoningEffort::Medium)),
            Self::High => Reasoning::Enabled(Some(ReasoningEffort::High)),
            Self::XHigh => Reasoning::Enabled(Some(ReasoningEffort::XHigh)),
            Self::Max => Reasoning::Enabled(Some(ReasoningEffort::Max)),
        }
    }
}

/// `claude-fable-5-1`.
pub const FABLE_5_1: ModelPreset<FableThinking> = ModelPreset::new(
    ApiDialect::Messages,
    BASE_URL,
    "claude-fable-5-1",
    API_KEY_ENV,
);

/// `claude-opus-5-5`.
pub const OPUS_5_5: ModelPreset<OpusThinking> = ModelPreset::new(
    ApiDialect::Messages,
    BASE_URL,
    "claude-opus-5-5",
    API_KEY_ENV,
);

/// `claude-sonnet-5-5`.
pub const SONNET_5_5: ModelPreset<SonnetThinking> = ModelPreset::new(
    ApiDialect::Messages,
    BASE_URL,
    "claude-sonnet-5-5",
    API_KEY_ENV,
);

/// `claude-haiku-5-5`.
pub const HAIKU_5_5: ModelPreset<HaikuThinking> = ModelPreset::new(
    ApiDialect::Messages,
    BASE_URL,
    "claude-haiku-5-5",
    API_KEY_ENV,
);
