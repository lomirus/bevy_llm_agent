pub mod agent;
mod app_ext;
mod chat_completion;
pub mod messages;
mod plugin;
pub mod tool;

pub use crate::app_ext::AppExt;
pub use crate::plugin::LlmAgentPlugin;

pub const DEEPSEEK_V4_FLASH: &str = "deepseek-v4-flash";
pub const DEEPSEEK_V4_PRO: &str = "deepseek-v4-pro";
