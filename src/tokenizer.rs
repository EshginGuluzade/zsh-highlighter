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

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Default,
    InWord,
    InSingleQuote,
    InDoubleQuote,
    InAnsiCQuote,
    InBacktick,
    InComment,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut state = State::Default;
    let mut token_start: usize = 0;
    let mut i: usize = 0;

    while i < len {
        match state {
            State::Default => {
                let b = bytes[i];

                // Skip whitespace
                if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                    i += 1;
                    continue;
                }

                // Line continuation: backslash + newline
                if b == b'\\' && i + 1 < len && bytes[i + 1] == b'\n' {
                    i += 2;
                    continue;
                }

                // Comment: # at start or after whitespace
                if b == b'#' {
                    let is_comment = i == 0
                        || matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r');
                    if is_comment {
                        token_start = i;
                        state = State::InComment;
                        i += 1;
                        continue;
                    }
                }

                // ANSI-C string: $'
                if b == b'$' && i + 1 < len && bytes[i + 1] == b'\'' {
                    token_start = i;
                    state = State::InAnsiCQuote;
                    i += 2; // skip $'
                    continue;
                }

                // Single-quoted string
                if b == b'\'' {
                    token_start = i;
                    state = State::InSingleQuote;
                    i += 1;
                    continue;
                }

                // Double-quoted string
                if b == b'"' {
                    token_start = i;
                    state = State::InDoubleQuote;
                    i += 1;
                    continue;
                }

                // Backtick string
                if b == b'`' {
                    token_start = i;
                    state = State::InBacktick;
                    i += 1;
                    continue;
                }

                // Operators (greedy matching)
                if let Some((op_len, _)) = match_operator(bytes, i, len) {
                    tokens.push(Token {
                        start: i,
                        end: i + op_len,
                        token_type: TokenType::Operator,
                    });
                    i += op_len;
                    continue;
                }

                // Start a word
                token_start = i;
                state = State::InWord;
                i += 1;
            }

            State::InWord => {
                if i >= len {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    break;
                }

                let b = bytes[i];

                // Line continuation inside word: acts as whitespace, breaks the word
                if b == b'\\' && i + 1 < len && bytes[i + 1] == b'\n' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    state = State::Default;
                    i += 2;
                    continue;
                }

                // Whitespace ends word
                if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    state = State::Default;
                    i += 1;
                    continue;
                }

                // Operator breaks word
                if match_operator(bytes, i, len).is_some() {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    state = State::Default;
                    // Don't advance i — let Default handle the operator
                    continue;
                }

                // String opener breaks word? No — in zsh, quotes mid-word are part of the word.
                // But per the spec, string openers start new tokens. Let's check if this is
                // at a word boundary... Actually, the spec says adjacency works when operators
                // or string boundaries separate tokens. So strings do break words.

                // Single quote starts string
                if b == b'\'' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    token_start = i;
                    state = State::InSingleQuote;
                    i += 1;
                    continue;
                }

                // Double quote starts string
                if b == b'"' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    token_start = i;
                    state = State::InDoubleQuote;
                    i += 1;
                    continue;
                }

                // Backtick starts string
                if b == b'`' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    token_start = i;
                    state = State::InBacktick;
                    i += 1;
                    continue;
                }

                // ANSI-C string $'
                if b == b'$' && i + 1 < len && bytes[i + 1] == b'\'' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Word { command_position: false },
                    });
                    token_start = i;
                    state = State::InAnsiCQuote;
                    i += 2;
                    continue;
                }

                // Comment # after word if preceded by whitespace — but we're InWord,
                // so # is just part of the word (e.g., foo#bar)
                // No special handling needed.

                i += 1;
            }

            State::InSingleQuote => {
                if i >= len {
                    // Unterminated — extend to end of input
                    tokens.push(Token {
                        start: token_start,
                        end: len,
                        token_type: TokenType::String,
                    });
                    break;
                }
                if bytes[i] == b'\'' {
                    tokens.push(Token {
                        start: token_start,
                        end: i + 1,
                        token_type: TokenType::String,
                    });
                    state = State::Default;
                    i += 1;
                } else {
                    i += 1;
                }
            }

            State::InDoubleQuote => {
                if i >= len {
                    tokens.push(Token {
                        start: token_start,
                        end: len,
                        token_type: TokenType::String,
                    });
                    break;
                }
                let b = bytes[i];
                if b == b'\\' && i + 1 < len && bytes[i + 1] == b'"' {
                    i += 2; // skip escaped quote
                } else if b == b'"' {
                    tokens.push(Token {
                        start: token_start,
                        end: i + 1,
                        token_type: TokenType::String,
                    });
                    state = State::Default;
                    i += 1;
                } else {
                    i += 1;
                }
            }

            State::InAnsiCQuote => {
                if i >= len {
                    tokens.push(Token {
                        start: token_start,
                        end: len,
                        token_type: TokenType::String,
                    });
                    break;
                }
                let b = bytes[i];
                if b == b'\\' && i + 1 < len && bytes[i + 1] == b'\'' {
                    i += 2; // skip escaped quote
                } else if b == b'\'' {
                    tokens.push(Token {
                        start: token_start,
                        end: i + 1,
                        token_type: TokenType::String,
                    });
                    state = State::Default;
                    i += 1;
                } else {
                    i += 1;
                }
            }

            State::InBacktick => {
                if i >= len {
                    tokens.push(Token {
                        start: token_start,
                        end: len,
                        token_type: TokenType::String,
                    });
                    break;
                }
                if bytes[i] == b'`' {
                    tokens.push(Token {
                        start: token_start,
                        end: i + 1,
                        token_type: TokenType::String,
                    });
                    state = State::Default;
                    i += 1;
                } else {
                    i += 1;
                }
            }

            State::InComment => {
                if i >= len {
                    tokens.push(Token {
                        start: token_start,
                        end: len,
                        token_type: TokenType::Comment,
                    });
                    break;
                }
                if bytes[i] == b'\n' {
                    tokens.push(Token {
                        start: token_start,
                        end: i,
                        token_type: TokenType::Comment,
                    });
                    state = State::Default;
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }
    }

    // Flush any pending token at end of input
    match state {
        State::InWord => {
            tokens.push(Token {
                start: token_start,
                end: len,
                token_type: TokenType::Word { command_position: false },
            });
        }
        State::InSingleQuote
        | State::InDoubleQuote
        | State::InAnsiCQuote
        | State::InBacktick => {
            tokens.push(Token {
                start: token_start,
                end: len,
                token_type: TokenType::String,
            });
        }
        State::InComment => {
            tokens.push(Token {
                start: token_start,
                end: len,
                token_type: TokenType::Comment,
            });
        }
        State::Default => {}
    }

    tokens
}

/// Match an operator at position `i`. Returns (length, operator_str) or None.
fn match_operator(bytes: &[u8], i: usize, len: usize) -> Option<(usize, &'static str)> {
    let b = bytes[i];
    match b {
        b'|' => {
            if i + 1 < len {
                if bytes[i + 1] == b'|' {
                    return Some((2, "||"));
                }
                if bytes[i + 1] == b'&' {
                    return Some((2, "|&"));
                }
            }
            Some((1, "|"))
        }
        b'&' => {
            if i + 1 < len && bytes[i + 1] == b'&' {
                return Some((2, "&&"));
            }
            Some((1, "&"))
        }
        b';' => {
            if i + 1 < len && bytes[i + 1] == b';' {
                return Some((2, ";;"));
            }
            Some((1, ";"))
        }
        b'>' => {
            if i + 1 < len && bytes[i + 1] == b'>' {
                return Some((2, ">>"));
            }
            Some((1, ">"))
        }
        b'<' => {
            if i + 2 < len && bytes[i + 1] == b'<' && bytes[i + 2] == b'<' {
                return Some((3, "<<<"));
            }
            if i + 1 < len && bytes[i + 1] == b'<' {
                return Some((2, "<<"));
            }
            Some((1, "<"))
        }
        b'(' => Some((1, "(")),
        b')' => Some((1, ")")),
        b'{' => Some((1, "{")),
        b'}' => Some((1, "}")),
        b'[' => {
            if i + 1 < len && bytes[i + 1] == b'[' {
                return Some((2, "[["));
            }
            None // single [ is part of a word
        }
        b']' => {
            if i + 1 < len && bytes[i + 1] == b']' {
                return Some((2, "]]"));
            }
            None // single ] is part of a word
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(start: usize, end: usize) -> Token {
        Token { start, end, token_type: TokenType::Word { command_position: false } }
    }

    fn string(start: usize, end: usize) -> Token {
        Token { start, end, token_type: TokenType::String }
    }

    fn comment(start: usize, end: usize) -> Token {
        Token { start, end, token_type: TokenType::Comment }
    }

    fn operator(start: usize, end: usize) -> Token {
        Token { start, end, token_type: TokenType::Operator }
    }

    // === Empty / whitespace ===

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize(""), Vec::new());
    }

    #[test]
    fn test_whitespace_only() {
        assert_eq!(tokenize("   \t\n  "), Vec::new());
    }

    // === Basic word splitting ===

    #[test]
    fn test_single_word() {
        assert_eq!(tokenize("ls"), vec![word(0, 2)]);
    }

    #[test]
    fn test_three_words() {
        assert_eq!(
            tokenize("ls -la /tmp"),
            vec![word(0, 2), word(3, 6), word(7, 11)]
        );
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        assert_eq!(tokenize("  ls  "), vec![word(2, 4)]);
    }

    // === Single-quoted strings ===

    #[test]
    fn test_single_quoted_string() {
        // echo 'hello world'
        assert_eq!(
            tokenize("echo 'hello world'"),
            vec![word(0, 4), string(5, 18)]
        );
    }

    #[test]
    fn test_unterminated_single_quote() {
        assert_eq!(
            tokenize("echo 'hello"),
            vec![word(0, 4), string(5, 11)]
        );
    }

    // === Double-quoted strings ===

    #[test]
    fn test_double_quoted_string() {
        // echo "hello"
        assert_eq!(
            tokenize("echo \"hello\""),
            vec![word(0, 4), string(5, 12)]
        );
    }

    #[test]
    fn test_double_quoted_with_escape() {
        // echo "he\"llo"
        assert_eq!(
            tokenize("echo \"he\\\"llo\""),
            vec![word(0, 4), string(5, 14)]
        );
    }

    #[test]
    fn test_unterminated_double_quote() {
        assert_eq!(
            tokenize("echo \"hello"),
            vec![word(0, 4), string(5, 11)]
        );
    }

    // === ANSI-C strings ===

    #[test]
    fn test_ansi_c_string() {
        // echo $'hello'
        assert_eq!(
            tokenize("echo $'hello'"),
            vec![word(0, 4), string(5, 13)]
        );
    }

    #[test]
    fn test_ansi_c_string_with_escape() {
        // echo $'he\'llo'
        assert_eq!(
            tokenize("echo $'he\\'llo'"),
            vec![word(0, 4), string(5, 15)]
        );
    }

    #[test]
    fn test_unterminated_ansi_c_string() {
        assert_eq!(
            tokenize("echo $'hello"),
            vec![word(0, 4), string(5, 12)]
        );
    }

    #[test]
    fn test_dollar_not_followed_by_single_quote() {
        // $var is just a word
        assert_eq!(tokenize("$var"), vec![word(0, 4)]);
    }

    // === Backtick strings ===

    #[test]
    fn test_backtick_string() {
        // echo `uname`
        assert_eq!(
            tokenize("echo `uname`"),
            vec![word(0, 4), string(5, 12)]
        );
    }

    #[test]
    fn test_unterminated_backtick() {
        assert_eq!(
            tokenize("echo `uname"),
            vec![word(0, 4), string(5, 11)]
        );
    }

    // === Comments ===

    #[test]
    fn test_standalone_comment() {
        assert_eq!(
            tokenize("# this is a comment"),
            vec![comment(0, 19)]
        );
    }

    #[test]
    fn test_inline_comment() {
        assert_eq!(
            tokenize("echo hello # comment"),
            vec![word(0, 4), word(5, 10), comment(11, 20)]
        );
    }

    #[test]
    fn test_hash_inside_word_not_comment() {
        assert_eq!(
            tokenize("foo#bar"),
            vec![word(0, 7)]
        );
    }

    #[test]
    fn test_hash_inside_single_quote_not_comment() {
        assert_eq!(
            tokenize("'hello # world'"),
            vec![string(0, 15)]
        );
    }

    #[test]
    fn test_hash_inside_double_quote_not_comment() {
        assert_eq!(
            tokenize("\"hello # world\""),
            vec![string(0, 15)]
        );
    }

    #[test]
    fn test_comment_after_operator() {
        // After a semicolon, # starts a comment (preceded by whitespace after ;)
        // Actually: ; is operator, then space, then #
        assert_eq!(
            tokenize("echo hi; # done"),
            vec![word(0, 4), word(5, 7), operator(7, 8), comment(9, 15)]
        );
    }

    // === Operators ===

    #[test]
    fn test_pipe() {
        assert_eq!(
            tokenize("ls | grep"),
            vec![word(0, 2), operator(3, 4), word(5, 9)]
        );
    }

    #[test]
    fn test_double_pipe() {
        assert_eq!(
            tokenize("a || b"),
            vec![word(0, 1), operator(2, 4), word(5, 6)]
        );
    }

    #[test]
    fn test_pipe_ampersand() {
        assert_eq!(
            tokenize("a |& b"),
            vec![word(0, 1), operator(2, 4), word(5, 6)]
        );
    }

    #[test]
    fn test_double_ampersand() {
        assert_eq!(
            tokenize("a && b"),
            vec![word(0, 1), operator(2, 4), word(5, 6)]
        );
    }

    #[test]
    fn test_semicolon() {
        assert_eq!(
            tokenize("a; b"),
            vec![word(0, 1), operator(1, 2), word(3, 4)]
        );
    }

    #[test]
    fn test_double_semicolon() {
        assert_eq!(
            tokenize("a;; b"),
            vec![word(0, 1), operator(1, 3), word(4, 5)]
        );
    }

    #[test]
    fn test_redirect_right() {
        assert_eq!(
            tokenize("echo > file"),
            vec![word(0, 4), operator(5, 6), word(7, 11)]
        );
    }

    #[test]
    fn test_redirect_append() {
        assert_eq!(
            tokenize("echo >> file"),
            vec![word(0, 4), operator(5, 7), word(8, 12)]
        );
    }

    #[test]
    fn test_redirect_left() {
        assert_eq!(
            tokenize("cat < file"),
            vec![word(0, 3), operator(4, 5), word(6, 10)]
        );
    }

    #[test]
    fn test_heredoc() {
        assert_eq!(
            tokenize("cat << EOF"),
            vec![word(0, 3), operator(4, 6), word(7, 10)]
        );
    }

    #[test]
    fn test_herestring() {
        assert_eq!(
            tokenize("cat <<< word"),
            vec![word(0, 3), operator(4, 7), word(8, 12)]
        );
    }

    #[test]
    fn test_background_ampersand() {
        assert_eq!(
            tokenize("sleep 10 &"),
            vec![word(0, 5), word(6, 8), operator(9, 10)]
        );
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(
            tokenize("(echo hi)"),
            vec![operator(0, 1), word(1, 5), word(6, 8), operator(8, 9)]
        );
    }

    // === Braces ===

    #[test]
    fn test_braces() {
        assert_eq!(
            tokenize("{ echo hi; }"),
            vec![operator(0, 1), word(2, 6), word(7, 9), operator(9, 10), operator(11, 12)]
        );
    }

    // === Double brackets ===

    #[test]
    fn test_double_brackets() {
        assert_eq!(
            tokenize("[[ -f file ]]"),
            vec![operator(0, 2), word(3, 5), word(6, 10), operator(11, 13)]
        );
    }

    #[test]
    fn test_single_bracket_is_word() {
        assert_eq!(
            tokenize("[ -f file ]"),
            vec![word(0, 1), word(2, 4), word(5, 9), word(10, 11)]
        );
    }

    // === Line continuation ===

    #[test]
    fn test_line_continuation_between_words() {
        assert_eq!(
            tokenize("echo\\\nhello"),
            vec![word(0, 4), word(6, 11)]
        );
    }

    #[test]
    fn test_line_continuation_in_whitespace() {
        assert_eq!(
            tokenize("echo \\\n hello"),
            vec![word(0, 4), word(8, 13)]
        );
    }

    // === Adjacency ===

    #[test]
    fn test_adjacent_word_operator_word() {
        assert_eq!(
            tokenize("echo>file"),
            vec![word(0, 4), operator(4, 5), word(5, 9)]
        );
    }

    #[test]
    fn test_adjacent_word_string() {
        assert_eq!(
            tokenize("echo\"hello\""),
            vec![word(0, 4), string(4, 11)]
        );
    }

    // === Operator greediness ===

    #[test]
    fn test_operator_greediness_pipe() {
        // || should be matched, not two |
        let tokens = tokenize("a || b");
        assert_eq!(tokens[1], operator(2, 4));
    }

    #[test]
    fn test_operator_greediness_redirect() {
        // >> should be matched, not two >
        let tokens = tokenize("echo >> file");
        assert_eq!(tokens[1], operator(5, 7));
    }

    #[test]
    fn test_operator_greediness_herestring() {
        // <<< should be matched, not << + <
        let tokens = tokenize("cat <<< word");
        assert_eq!(tokens[1], operator(4, 7));
    }

    #[test]
    fn test_operator_greediness_semicolon() {
        // ;; should be matched, not two ;
        let tokens = tokenize("a;; b");
        assert_eq!(tokens[1], operator(1, 3));
    }
}
