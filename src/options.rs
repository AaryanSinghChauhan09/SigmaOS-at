//! Options model.

#[derive(Debug, Clone)]
pub struct Options {
    pub verbose: bool,
}

impl Default for Options {
    fn default() -> Self { Self { verbose: false } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opts() { assert!(!Options::default().verbose); }
}
