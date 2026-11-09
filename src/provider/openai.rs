//! OpenAI models, served through the Responses API, which they need for tool calling while
//! reasoning.

use super::{ModelPreset, Reasoning, ReasoningEffort, ReasoningFormat, Sealed, Thinking};
use crate::agent::ApiDialect;

const BASE_URL: &str = "https://api.openai.com/v1";
const API_KEY_ENV: &str = "OPENAI_API_KEY";

/// Thinking options for [`GPT_6_ASTRA`] and [`GPT_6_1_SOL`], which always reason.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AstraThinking {
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
}

impl Thinking for AstraThinking {}

impl Sealed for AstraThinking {
    const REASONING_FORMAT: ReasoningFormat = ReasoningFormat::Encrypted;

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

/// Thinking options for [`GPT_6_1_TERRA`] and [`GPT_6_LUNA`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TerraThinking {
    Off,
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
}

impl Thinking for TerraThinking {}

impl Sealed for TerraThinking {
    const REASONING_FORMAT: ReasoningFormat = ReasoningFormat::Encrypted;

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

/// `gpt-6-astra`.
pub const GPT_6_ASTRA: ModelPreset<AstraThinking> =
    ModelPreset::new(ApiDialect::Responses, BASE_URL, "gpt-6-astra", API_KEY_ENV);

/// `gpt-6.1-sol`.
pub const GPT_6_1_SOL: ModelPreset<AstraThinking> =
    ModelPreset::new(ApiDialect::Responses, BASE_URL, "gpt-6.1-sol", API_KEY_ENV);

/// `gpt-6.1-terra`.
pub const GPT_6_1_TERRA: ModelPreset<TerraThinking> = ModelPreset::new(
    ApiDialect::Responses,
    BASE_URL,
    "gpt-6.1-terra",
    API_KEY_ENV,
);

/// `gpt-6-luna`.
pub const GPT_6_LUNA: ModelPreset<TerraThinking> =
    ModelPreset::new(ApiDialect::Responses, BASE_URL, "gpt-6-luna", API_KEY_ENV);
