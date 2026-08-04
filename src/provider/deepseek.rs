use super::ProviderBuilder;
use crate::agent::{ApiDialect, Thinking};

const BASE_URL: &str = "https://api.deepseek.com";
const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
const FLASH_SUPPORTED_THINKING: &[Thinking] =
    &[Thinking::Off, Thinking::Low, Thinking::High, Thinking::Max];
const PRO_SUPPORTED_THINKING: &[Thinking] = &[Thinking::Off, Thinking::High, Thinking::Max];

pub const V4_FLASH: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::DeepSeekResponses,
    base_url: BASE_URL,
    model: "deepseek-v4-flash",
    api_key_env: API_KEY_ENV,
    supported_thinking: FLASH_SUPPORTED_THINKING,
};
pub const V4_PRO: ProviderBuilder<'static> = ProviderBuilder {
    api_dialect: ApiDialect::DeepSeekChatCompletions,
    base_url: BASE_URL,
    model: "deepseek-v4-pro",
    api_key_env: API_KEY_ENV,
    supported_thinking: PRO_SUPPORTED_THINKING,
};
