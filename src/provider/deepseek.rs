use super::Provider;

const BASE_URL: &str = "https://api.deepseek.com";
const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

pub const V4_FLASH: Provider<'static> = Provider::new(BASE_URL, "deepseek-v4-flash", API_KEY_ENV);
pub const V4_PRO: Provider<'static> = Provider::new(BASE_URL, "deepseek-v4-pro", API_KEY_ENV);
