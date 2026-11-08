use super::{ModelPreset, Reasoning, ReasoningEffort, Sealed, Thinking};
use crate::agent::ApiDialect;

const BASE_URL: &str = "https://api.deepseek.com";
const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

/// Thinking options for [`V4_FLASH`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FlashThinking {
    Off,
    Low,
    #[default]
    High,
    Max,
}

impl Thinking for FlashThinking {}

impl Sealed for FlashThinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::Disabled,
            Self::Low => Reasoning::Enabled(Some(ReasoningEffort::Low)),
            Self::High => Reasoning::Enabled(Some(ReasoningEffort::High)),
            Self::Max => Reasoning::Enabled(Some(ReasoningEffort::Max)),
        }
    }
}

/// Thinking options for [`V4_PRO`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ProThinking {
    Off,
    #[default]
    High,
    Max,
}

impl Thinking for ProThinking {}

impl Sealed for ProThinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::Disabled,
            Self::High => Reasoning::Enabled(Some(ReasoningEffort::High)),
            Self::Max => Reasoning::Enabled(Some(ReasoningEffort::Max)),
        }
    }
}

/// `deepseek-v4-flash`.
pub const V4_FLASH: ModelPreset<'static, FlashThinking> = ModelPreset::new(
    ApiDialect::DeepSeekResponses,
    BASE_URL,
    "deepseek-v4-flash",
    API_KEY_ENV,
);

/// `deepseek-v4-pro`.
pub const V4_PRO: ModelPreset<'static, ProThinking> = ModelPreset::new(
    ApiDialect::DeepSeekChatCompletions,
    BASE_URL,
    "deepseek-v4-pro",
    API_KEY_ENV,
);
