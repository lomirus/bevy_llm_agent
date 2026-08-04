use super::ProviderBuilder;
use crate::agent::{ApiDialect, Thinking};

const GLOBAL_BASE_URL: &str = "https://api.moonshot.ai/v1";
const CHINA_BASE_URL: &str = "https://api.moonshot.cn/v1";
const API_KEY_ENV: &str = "MOONSHOT_API_KEY";
const K3_SUPPORTED_THINKING: &[Thinking] = &[Thinking::Low, Thinking::High, Thinking::Max];
const K2_5_SUPPORTED_THINKING: &[Thinking] = &[Thinking::Off, Thinking::High];
const K2_6_SUPPORTED_THINKING: &[Thinking] = &[Thinking::Off, Thinking::High];
const K2_7_SUPPORTED_THINKING: &[Thinking] = &[Thinking::High];

/// `kimi-k3`.
///
/// Allowed [`Thinking`] options: [`Low`](Thinking::Low), [`High`](Thinking::High), [`Max`](Thinking::Max).
pub const K3: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::MoonshotChatCompletions,
    base_url: GLOBAL_BASE_URL,
    model: "kimi-k3",
    api_key_env: API_KEY_ENV,
    supported_thinking: K3_SUPPORTED_THINKING,
};

/// `kimi-k2.7-code`.
///
/// Allowed [`Thinking`] option: [`High`](Thinking::High).
pub const K2_7_CODE: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::MoonshotChatCompletions,
    base_url: GLOBAL_BASE_URL,
    model: "kimi-k2.7-code",
    api_key_env: API_KEY_ENV,
    supported_thinking: K2_7_SUPPORTED_THINKING,
};

/// `kimi-k2.7-code-highspeed`.
///
/// Allowed [`Thinking`] option: [`High`](Thinking::High).
pub const K2_7_CODE_HIGHSPEED: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::MoonshotChatCompletions,
    base_url: GLOBAL_BASE_URL,
    model: "kimi-k2.7-code-highspeed",
    api_key_env: API_KEY_ENV,
    supported_thinking: K2_7_SUPPORTED_THINKING,
};

/// `kimi-k2.6`.
///
/// Allowed [`Thinking`] options: [`Off`](Thinking::Off), [`High`](Thinking::High).
pub const K2_6: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::MoonshotChatCompletions,
    base_url: GLOBAL_BASE_URL,
    model: "kimi-k2.6",
    api_key_env: API_KEY_ENV,
    supported_thinking: K2_6_SUPPORTED_THINKING,
};

/// `kimi-k2.5`.
///
/// Allowed [`Thinking`] options: [`Off`](Thinking::Off), [`High`](Thinking::High).
pub const K2_5: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::MoonshotChatCompletions,
    base_url: GLOBAL_BASE_URL,
    model: "kimi-k2.5",
    api_key_env: API_KEY_ENV,
    supported_thinking: K2_5_SUPPORTED_THINKING,
};

/// China API endpoint presets.
pub mod china {
    use super::{CHINA_BASE_URL, ProviderBuilder};

    pub const K3: ProviderBuilder<'static> = ProviderBuilder {
        base_url: CHINA_BASE_URL,
        ..super::K3
    };
    pub const K2_7_CODE: ProviderBuilder<'static> = ProviderBuilder {
        base_url: CHINA_BASE_URL,
        ..super::K2_7_CODE
    };
    pub const K2_7_CODE_HIGHSPEED: ProviderBuilder<'static> = ProviderBuilder {
        base_url: CHINA_BASE_URL,
        ..super::K2_7_CODE_HIGHSPEED
    };
    pub const K2_6: ProviderBuilder<'static> = ProviderBuilder {
        base_url: CHINA_BASE_URL,
        ..super::K2_6
    };
    pub const K2_5: ProviderBuilder<'static> = ProviderBuilder {
        base_url: CHINA_BASE_URL,
        ..super::K2_5
    };
}
