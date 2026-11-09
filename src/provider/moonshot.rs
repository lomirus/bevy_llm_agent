use super::{ModelPreset, Reasoning, ReasoningEffort, Sealed, Thinking};
use crate::agent::ApiDialect;

const GLOBAL_BASE_URL: &str = "https://api.moonshot.ai/v1";
const CHINA_BASE_URL: &str = "https://api.moonshot.cn/v1";
const API_KEY_ENV: &str = "MOONSHOT_API_KEY";

/// Thinking options for [`K3`], [`K3_1_PRO`], and [`K3_1_PRO_HIGHSPEED`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum K3Thinking {
    Low,
    High,
    #[default]
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
pub enum K2_7Thinking {
    #[default]
    High,
}

impl Thinking for K2_7Thinking {}

impl Sealed for K2_7Thinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::High => Reasoning::Enabled(None),
        }
    }
}

/// Thinking options for [`K2_6`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum K2_6Thinking {
    Off,
    #[default]
    High,
}

impl Thinking for K2_6Thinking {}

impl Sealed for K2_6Thinking {
    fn reasoning(self) -> Reasoning {
        match self {
            Self::Off => Reasoning::Disabled,
            Self::High => Reasoning::Enabled(None),
        }
    }
}

/// `kimi-k3`.
pub const K3: ModelPreset<K3Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k3",
    API_KEY_ENV,
);

/// `kimi-k3.1-pro`.
pub const K3_1_PRO: ModelPreset<K3Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k3.1-pro",
    API_KEY_ENV,
);

/// `kimi-k3.1-pro-highspeed`.
pub const K3_1_PRO_HIGHSPEED: ModelPreset<K3Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k3.1-pro-highspeed",
    API_KEY_ENV,
);

/// `kimi-k2.7-code`.
pub const K2_7_CODE: ModelPreset<K2_7Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.7-code",
    API_KEY_ENV,
);

/// `kimi-k2.7-code-highspeed`.
pub const K2_7_CODE_HIGHSPEED: ModelPreset<K2_7Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.7-code-highspeed",
    API_KEY_ENV,
);

/// `kimi-k2.6`.
pub const K2_6: ModelPreset<K2_6Thinking> = ModelPreset::new(
    ApiDialect::ChatCompletions,
    GLOBAL_BASE_URL,
    "kimi-k2.6",
    API_KEY_ENV,
);

/// China API endpoint presets.
pub mod china {
    use super::{CHINA_BASE_URL, K2_6Thinking, K2_7Thinking, K3Thinking, ModelPreset};

    pub const K3: ModelPreset<K3Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K3
    };
    pub const K3_1_PRO: ModelPreset<K3Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K3_1_PRO
    };
    pub const K3_1_PRO_HIGHSPEED: ModelPreset<K3Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K3_1_PRO_HIGHSPEED
    };
    pub const K2_7_CODE: ModelPreset<K2_7Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_7_CODE
    };
    pub const K2_7_CODE_HIGHSPEED: ModelPreset<K2_7Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_7_CODE_HIGHSPEED
    };
    pub const K2_6: ModelPreset<K2_6Thinking> = ModelPreset {
        base_url: CHINA_BASE_URL,
        ..super::K2_6
    };
}
