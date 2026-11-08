use super::{ModelPreset, Reasoning, ReasoningEffort, Sealed, Thinking};
use crate::agent::ApiDialect;

const GLOBAL_BASE_URL: &str = "https://api.moonshot.ai/v1";
const CHINA_BASE_URL: &str = "https://api.moonshot.cn/v1";
const API_KEY_ENV: &str = "MOONSHOT_API_KEY";

/// Thinking options for [`K3`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum K3Thinking {
    Low,
    #[default]
    High,
    Max,
}

impl Thinking for K3Thinking {}

impl Sealed for K3Thinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Low => Reasoning::Enabled(Some(ReasoningEffort::Low)),
            Self::High => Reasoning::Enabled(Some(ReasoningEffort::High)),
            Self::Max => Reasoning::Enabled(Some(ReasoningEffort::Max)),
        }
    }
}

/// Thinking options for [`K2_7_CODE`] and [`K2_7_CODE_HIGHSPEED`], which always think.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum K2_7CodeThinking {
    #[default]
    High,
}

impl Thinking for K2_7CodeThinking {}

impl Sealed for K2_7CodeThinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::High => Reasoning::Enabled(None),
        }
    }
}

/// Thinking options for [`K2_6`] and [`K2_5`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum K2Thinking {
    #[default]
    Off,
    High,
}

impl Thinking for K2Thinking {}

impl Sealed for K2Thinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::Disabled,
            Self::High => Reasoning::Enabled(None),
        }
    }
}

/// `kimi-k3`.
pub const K3: ModelPreset<'static, K3Thinking> = ModelPreset::new(
    ApiDialect::MoonshotChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k3",
    API_KEY_ENV,
);

/// `kimi-k2.7-code`.
pub const K2_7_CODE: ModelPreset<'static, K2_7CodeThinking> = ModelPreset::new(
    ApiDialect::MoonshotChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.7-code",
    API_KEY_ENV,
);

/// `kimi-k2.7-code-highspeed`.
pub const K2_7_CODE_HIGHSPEED: ModelPreset<'static, K2_7CodeThinking> = ModelPreset::new(
    ApiDialect::MoonshotChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.7-code-highspeed",
    API_KEY_ENV,
);

/// `kimi-k2.6`.
pub const K2_6: ModelPreset<'static, K2Thinking> = ModelPreset::new(
    ApiDialect::MoonshotChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.6",
    API_KEY_ENV,
);

/// `kimi-k2.5`.
pub const K2_5: ModelPreset<'static, K2Thinking> = ModelPreset::new(
    ApiDialect::MoonshotChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.5",
    API_KEY_ENV,
);

/// China API endpoint presets.
pub mod china {
    use super::{CHINA_BASE_URL, K2_7CodeThinking, K2Thinking, K3Thinking, ModelPreset};

    pub const K3: ModelPreset<'static, K3Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K3
    };
    pub const K2_7_CODE: ModelPreset<'static, K2_7CodeThinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_7_CODE
    };
    pub const K2_7_CODE_HIGHSPEED: ModelPreset<'static, K2_7CodeThinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_7_CODE_HIGHSPEED
    };
    pub const K2_6: ModelPreset<'static, K2Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_6
    };
    pub const K2_5: ModelPreset<'static, K2Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_5
    };
}
