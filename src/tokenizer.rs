#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenType {
    Word { command_position: bool },
    String,
    Comment,
    Operator,
    ReservedWord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

pub fn tokenize(_input: &str) -> Vec<Token> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize(""), Vec::new());
    }

    #[test]
    fn test_returns_empty_vec() {
        assert!(tokenize("ls -la").is_empty());
    }
}
