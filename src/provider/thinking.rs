use std::{fmt, hash::Hash};

/// The thinking options accepted by a family of models.
///
/// Each [`ModelPreset`](super::ModelPreset) is tied to one implementor,
/// so only values the model accepts can be passed to
/// [`AgentBuilder::thinking`](crate::agent::AgentBuilder::thinking).
/// The [`Default`] value is the model's default thinking level.
///
/// This trait is sealed; its implementors live in the provider modules.
pub trait Thinking: Copy + Default + fmt::Debug + Eq + Hash + sealed::Sealed {}

/// Unnameable outside the crate: seals [`Thinking`] and hides the request parameters.
pub(crate) mod sealed {
    pub trait Sealed {
        fn reasoning(self) -> Reasoning;
    }

    /// Reasoning parameters sent to the API, resolved from a model's [`Thinking`](super::Thinking) option.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub enum Reasoning {
        #[default]
        Disabled,
        /// Enabled, with an optional effort level. `None` omits the effort from the request.
        Enabled(Option<ReasoningEffort>),
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ReasoningEffort {
        Low,
        High,
        Max,
    }

    impl ReasoningEffort {
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Low => "low",
                Self::High => "high",
                Self::Max => "max",
            }
        }
    }
}
