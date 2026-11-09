use super::{ModelPreset, Reasoning, ReasoningEffort, Sealed};
use crate::agent::ApiDialect;

const BASE_URL: &str = "https://api.deepseek.com";
const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

/// Thinking options for [`FLASH`] and [`V4_PRO`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Thinking {
    Off,
    Low,
    #[default]
    High,
    Max,
}

impl super::Thinking for Thinking {}

impl Sealed for Thinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::Disabled,
            Self::Low => Reasoning::Enabled(Some(ReasoningEffort::Low)),
            Self::High => Reasoning::Enabled(Some(ReasoningEffort::High)),
            Self::Max => Reasoning::Enabled(Some(ReasoningEffort::Max)),
        }
    }
}

/// `deepseek-flash`.
pub const FLASH: ModelPreset<Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    BASE_URL,
    "deepseek-flash",
    API_KEY_ENV,
);

/// `deepseek-v4-pro`.
pub const V4_PRO: ModelPreset<Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    BASE_URL,
    "deepseek-v4-pro",
    API_KEY_ENV,
);
