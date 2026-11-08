use super::ModelPreset;
use crate::agent::{ApiDialect, Thinking};

const BASE_URL: &str = "https://api.deepseek.com";
const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
const FLASH_SUPPORTED_THINKING: &[Thinking] =
    &[Thinking::Off, Thinking::Low, Thinking::High, Thinking::Max];
const PRO_SUPPORTED_THINKING: &[Thinking] = &[Thinking::Off, Thinking::High, Thinking::Max];

/// `deepseek-v4-flash`.
///
/// Allowed [`Thinking`] options: [`Off`](Thinking::Off), [`Low`](Thinking::Low), [`High`](Thinking::High), [`Max`](Thinking::Max).
pub const V4_FLASH: ModelPreset<'static> = ModelPreset {
    api_dialect: ApiDialect::DeepSeekResponses,
    base_url: BASE_URL,
    model: "deepseek-v4-flash",
    api_key_env: API_KEY_ENV,
    supported_thinking: FLASH_SUPPORTED_THINKING,
    default_thinking: Thinking::High,
};

/// `deepseek-v4-pro`.
///
/// Allowed [`Thinking`] options: [`Off`](Thinking::Off), [`High`](Thinking::High), [`Max`](Thinking::Max).
pub const V4_PRO: ModelPreset<'static> = ModelPreset {
    api_dialect: ApiDialect::DeepSeekChatCompletions,
    base_url: BASE_URL,
    model: "deepseek-v4-pro",
    api_key_env: API_KEY_ENV,
    supported_thinking: PRO_SUPPORTED_THINKING,
    default_thinking: Thinking::High,
};
