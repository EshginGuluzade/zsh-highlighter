use std::collections::HashSet;

use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub start: usize,
    pub end: usize,
    pub style: &'static str,
}

pub fn classify(_tokens: &[Token], _known_commands: &HashSet<&str>) -> Vec<Highlight> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tokens() {
        let commands: HashSet<&str> = HashSet::new();
        assert_eq!(classify(&[], &commands), Vec::new());
    }
}
