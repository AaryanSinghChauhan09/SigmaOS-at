//! SigmaosAt model.

#[derive(Debug, Clone)]
pub struct SigmaosAt {
    pub option: String,
}

impl Default for SigmaosAt {
    fn default() -> Self { Self { option: String::new() } }
}

impl SigmaosAt {
    pub fn new() -> Self { Self::default() }

    pub fn run(&self, target: &str) -> String {
        format!("{}:{}", self.option, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run() {
        let s = SigmaosAt::new();
        assert_eq!(s.run("target"), ":target");
    }
}
