use bevy::ecs::system::BoxedSystem;
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};

pub trait ToolTrait: 'static {
    type Args: DeserializeOwned + Send + Sync + JsonSchema;
    type Output: Serialize + Send;

    fn boxed_system() -> BoxedSystem;
}
