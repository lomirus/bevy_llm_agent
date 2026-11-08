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
        /// How models with these options expose reasoning.
        const REASONING_FORMAT: ReasoningFormat = ReasoningFormat::Plaintext;

        fn reasoning(self) -> Reasoning;
    }

    /// How a provider exposes reasoning, which decides the provider extensions in requests.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub enum ReasoningFormat {
        /// Reasoning is switched with a `thinking` parameter, streamed as plain text, and sent
        /// back as plain text. Used by DeepSeek and Moonshot.
        #[default]
        Plaintext,
        /// Reasoning is only tuned by its effort, and stateless requests get it encrypted to send
        /// back unchanged. Used by OpenAI.
        Encrypted,
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
        Medium,
        High,
        XHigh,
        Max,
    }

    impl ReasoningEffort {
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Low => "low",
                Self::Medium => "medium",
                Self::High => "high",
                Self::XHigh => "xhigh",
                Self::Max => "max",
            }
        }
    }
}
