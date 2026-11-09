//! Google's Gemini models, served statelessly through the Interactions API.
//!
//! Gemini models always think; reasoning streams as summaries.

use super::{ModelPreset, Reasoning, ReasoningEffort, Sealed, Thinking};
use crate::agent::ApiDialect;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const API_KEY_ENV: &str = "GEMINI_API_KEY";

/// Thinking options for [`GEMINI_4_ARGON`] and [`GEMINI_3_8_FLASH`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ArgonThinking {
    Low,
    #[default]
    Medium,
    High,
}

impl Thinking for ArgonThinking {}

impl Sealed for ArgonThinking {
    fn reasoning(self) -> Reasoning {
        Reasoning::Enabled(Some(match self {
            Self::Low => ReasoningEffort::Low,
            Self::Medium => ReasoningEffort::Medium,
            Self::High => ReasoningEffort::High,
        }))
    }
}

/// Thinking options for [`GEMINI_3_6_FLASH`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Flash3_6Thinking {
    Minimal,
    Low,
    #[default]
    Medium,
    High,
}

impl Thinking for Flash3_6Thinking {}

impl Sealed for Flash3_6Thinking {
    fn reasoning(self) -> Reasoning {
        Reasoning::Enabled(Some(match self {
            Self::Minimal => ReasoningEffort::Minimal,
            Self::Low => ReasoningEffort::Low,
            Self::Medium => ReasoningEffort::Medium,
            Self::High => ReasoningEffort::High,
        }))
    }
}

/// Thinking options for [`GEMINI_3_5_FLASH_LITE`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FlashLite3_5Thinking {
    #[default]
    Minimal,
    Low,
    Medium,
    High,
}

impl Thinking for FlashLite3_5Thinking {}

impl Sealed for FlashLite3_5Thinking {
    fn reasoning(self) -> Reasoning {
        Reasoning::Enabled(Some(match self {
            Self::Minimal => ReasoningEffort::Minimal,
            Self::Low => ReasoningEffort::Low,
            Self::Medium => ReasoningEffort::Medium,
            Self::High => ReasoningEffort::High,
        }))
    }
}

/// `gemini-4-argon`.
pub const GEMINI_4_ARGON: ModelPreset<ArgonThinking> = ModelPreset::new(
    ApiDialect::Interactions,
    BASE_URL,
    "gemini-4-argon",
    API_KEY_ENV,
);

/// `gemini-3.8-flash`.
pub const GEMINI_3_8_FLASH: ModelPreset<ArgonThinking> = ModelPreset::new(
    ApiDialect::Interactions,
    BASE_URL,
    "gemini-3.8-flash",
    API_KEY_ENV,
);

/// `gemini-3.6-flash`.
pub const GEMINI_3_6_FLASH: ModelPreset<Flash3_6Thinking> = ModelPreset::new(
    ApiDialect::Interactions,
    BASE_URL,
    "gemini-3.6-flash",
    API_KEY_ENV,
);

/// `gemini-3.5-flash-lite`.
pub const GEMINI_3_5_FLASH_LITE: ModelPreset<FlashLite3_5Thinking> = ModelPreset::new(
    ApiDialect::Interactions,
    BASE_URL,
    "gemini-3.5-flash-lite",
    API_KEY_ENV,
);
