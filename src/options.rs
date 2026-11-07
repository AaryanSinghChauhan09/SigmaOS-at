//! Options model.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Default,
    List,
    Remove(u64),
    Queue(String),
}

#[derive(Debug, Clone)]
pub struct Options {
    pub mode: Mode,
    pub command: Vec<String>,
}

impl Default for Options {
    fn default() -> Self { Self { mode: Mode::Default, command: vec![] } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opts() { assert_eq!(Options::default().mode, Mode::Default); }
}
