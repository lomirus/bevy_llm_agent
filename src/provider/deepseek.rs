use super::Provider;
use crate::agent::ApiDialect;

const BASE_URL: &str = "https://api.deepseek.com";
const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

pub const V4_FLASH: Provider<'static> = Provider::new(
    ApiDialect::DeepSeekChatCompletions,
    BASE_URL,
    "deepseek-v4-flash",
    API_KEY_ENV,
);
pub const V4_PRO: Provider<'static> = Provider::new(
    ApiDialect::DeepSeekChatCompletions,
    BASE_URL,
    "deepseek-v4-pro",
    API_KEY_ENV,
);
