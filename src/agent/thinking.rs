#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Thinking {
    #[default]
    Off,
    Low,
    High,
    Max,
}
