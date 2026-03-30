use std::collections::HashSet;

use crate::tokenizer::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub start: usize,
    pub end: usize,
    pub style: &'static str,
}

pub fn classify(tokens: &[Token], input: &str, known_commands: &HashSet<&str>) -> Vec<Highlight> {
    let mut highlights = Vec::new();

    for token in tokens {
        let style = match &token.token_type {
            TokenType::Word { command_position: true } => {
                if known_commands.is_empty() {
                    "fg=green,bold"
                } else {
                    let word = &input[token.start..token.end];
                    if known_commands.contains(word) {
                        "fg=green,bold"
                    } else {
                        "fg=red,underline"
                    }
                }
            }
            TokenType::Word { command_position: false } => continue,
            TokenType::ReservedWord => "fg=yellow,bold",
            TokenType::String => "fg=yellow",
            TokenType::Comment => "fg=8",
            TokenType::Operator => "fg=cyan",
        };

        highlights.push(Highlight {
            start: token.start,
            end: token.end,
            style,
        });
    }

    highlights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{Token, TokenType};

    fn make_word(start: usize, end: usize, cmd: bool) -> Token {
        Token { start, end, token_type: TokenType::Word { command_position: cmd } }
    }

    fn make_token(start: usize, end: usize, tt: TokenType) -> Token {
        Token { start, end, token_type: tt }
    }

    #[test]
    fn test_empty_tokens() {
        let commands: HashSet<&str> = HashSet::new();
        assert_eq!(classify(&[], "", &commands), Vec::new());
    }

    #[test]
    fn test_valid_command() {
        let commands: HashSet<&str> = ["ls"].into_iter().collect();
        let tokens = vec![make_word(0, 2, true)];
        let result = classify(&tokens, "ls", &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 2, style: "fg=green,bold" }]);
    }

    #[test]
    fn test_invalid_command() {
        let commands: HashSet<&str> = ["git", "ls"].into_iter().collect();
        let tokens = vec![make_word(0, 3, true)];
        let result = classify(&tokens, "gti", &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 3, style: "fg=red,underline" }]);
    }

    #[test]
    fn test_arguments_unstyled() {
        let commands: HashSet<&str> = ["ls"].into_iter().collect();
        let tokens = vec![
            make_word(0, 2, true),
            make_word(3, 6, false),
            make_word(7, 11, false),
        ];
        let result = classify(&tokens, "ls -la /tmp", &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 2, style: "fg=green,bold" }]);
    }

    #[test]
    fn test_empty_known_commands_defaults_valid() {
        let commands: HashSet<&str> = HashSet::new();
        let tokens = vec![make_word(0, 3, true)];
        let result = classify(&tokens, "foo", &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 3, style: "fg=green,bold" }]);
    }

    #[test]
    fn test_reserved_word() {
        let tokens = vec![make_token(0, 2, TokenType::ReservedWord)];
        let commands: HashSet<&str> = HashSet::new();
        let result = classify(&tokens, "if", &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 2, style: "fg=yellow,bold" }]);
    }

    #[test]
    fn test_string() {
        let tokens = vec![make_token(5, 18, TokenType::String)];
        let commands: HashSet<&str> = HashSet::new();
        let result = classify(&tokens, "echo 'hello world'", &commands);
        assert_eq!(result, vec![Highlight { start: 5, end: 18, style: "fg=yellow" }]);
    }

    #[test]
    fn test_comment() {
        let tokens = vec![make_token(0, 12, TokenType::Comment)];
        let commands: HashSet<&str> = HashSet::new();
        let result = classify(&tokens, "# a comment", &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 12, style: "fg=8" }]);
    }

    #[test]
    fn test_operator() {
        let tokens = vec![make_token(3, 4, TokenType::Operator)];
        let commands: HashSet<&str> = HashSet::new();
        let result = classify(&tokens, "ls | grep", &commands);
        assert_eq!(result, vec![Highlight { start: 3, end: 4, style: "fg=cyan" }]);
    }

    #[test]
    fn test_end_to_end_ls() {
        // 'ls -la /tmp' with known_commands={'ls'}
        // ls gets fg=green,bold; -la and /tmp are unstyled
        let commands: HashSet<&str> = ["ls"].into_iter().collect();
        let input = "ls -la /tmp";
        let mut tokens = crate::tokenizer::tokenize(input);
        crate::tokenizer::mark_command_positions(&mut tokens, input);
        let result = classify(&tokens, input, &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 2, style: "fg=green,bold" }]);
    }

    #[test]
    fn test_end_to_end_invalid_command() {
        // 'gti status' with known_commands={'git','ls'}
        let commands: HashSet<&str> = ["git", "ls"].into_iter().collect();
        let input = "gti status";
        let mut tokens = crate::tokenizer::tokenize(input);
        crate::tokenizer::mark_command_positions(&mut tokens, input);
        let result = classify(&tokens, input, &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 3, style: "fg=red,underline" }]);
    }

    #[test]
    fn test_end_to_end_if_then_fi() {
        // 'if true; then echo hi; fi' with known_commands={'true','echo'}
        let commands: HashSet<&str> = ["true", "echo"].into_iter().collect();
        let input = "if true; then echo hi; fi";
        let mut tokens = crate::tokenizer::tokenize(input);
        crate::tokenizer::mark_command_positions(&mut tokens, input);
        let result = classify(&tokens, input, &commands);
        // if(0,2) ReservedWord -> fg=yellow,bold
        // true(3,7) Word cmd=true -> fg=green,bold
        // ;(7,8) Operator -> fg=cyan
        // then(9,13) ReservedWord -> fg=yellow,bold
        // echo(14,18) Word cmd=true -> fg=green,bold
        // hi(19,21) Word cmd=false -> unstyled
        // ;(21,22) Operator -> fg=cyan
        // fi(23,25) ReservedWord -> fg=yellow,bold
        assert_eq!(result, vec![
            Highlight { start: 0, end: 2, style: "fg=yellow,bold" },
            Highlight { start: 3, end: 7, style: "fg=green,bold" },
            Highlight { start: 7, end: 8, style: "fg=cyan" },
            Highlight { start: 9, end: 13, style: "fg=yellow,bold" },
            Highlight { start: 14, end: 18, style: "fg=green,bold" },
            Highlight { start: 21, end: 22, style: "fg=cyan" },
            Highlight { start: 23, end: 25, style: "fg=yellow,bold" },
        ]);
    }

    #[test]
    fn test_end_to_end_echo_string() {
        // 'echo "hello world"' with known_commands={'echo'}
        let commands: HashSet<&str> = ["echo"].into_iter().collect();
        let input = "echo \"hello world\"";
        let mut tokens = crate::tokenizer::tokenize(input);
        crate::tokenizer::mark_command_positions(&mut tokens, input);
        let result = classify(&tokens, input, &commands);
        assert_eq!(result, vec![
            Highlight { start: 0, end: 4, style: "fg=green,bold" },
            Highlight { start: 5, end: 18, style: "fg=yellow" },
        ]);
    }

    #[test]
    fn test_end_to_end_comment() {
        // '# a comment'
        let commands: HashSet<&str> = HashSet::new();
        let input = "# a comment";
        let mut tokens = crate::tokenizer::tokenize(input);
        crate::tokenizer::mark_command_positions(&mut tokens, input);
        let result = classify(&tokens, input, &commands);
        assert_eq!(result, vec![Highlight { start: 0, end: 11, style: "fg=8" }]);
    }
}
