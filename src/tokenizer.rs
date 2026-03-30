#[derive(Debug, Clone, PartialEq)]
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

const RESERVED_WORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi",
    "for", "in", "while", "until", "do", "done",
    "case", "esac", "function", "select", "repeat", "time",
];

/// Second pass: mark command positions and identify reserved words.
/// Also restyles `{`, `}`, `[[`, `]]` operators as ReservedWord.
pub fn mark_command_positions(tokens: &mut [Token], input: &str) {
    let mut expect_command = true;
    let mut expect_in_keyword = false;

    for i in 0..tokens.len() {
        match &tokens[i].token_type {
            TokenType::Word { .. } => {
                let text = &input[tokens[i].start..tokens[i].end];

                // Check for `in` keyword after for/case/select variable
                if expect_in_keyword && text == "in" {
                    tokens[i].token_type = TokenType::ReservedWord;
                    expect_in_keyword = false;
                    expect_command = false;
                    continue;
                }

                if expect_command {
                    if text == "!" {
                        tokens[i].token_type = TokenType::ReservedWord;
                        expect_command = true;
                    } else if RESERVED_WORDS.contains(&text) {
                        tokens[i].token_type = TokenType::ReservedWord;
                        match text {
                            "then" | "else" | "elif" | "do" | "time"
                            | "if" | "while" | "until" => {
                                expect_command = true;
                            }
                            "for" | "case" | "select" => {
                                expect_command = false;
                                expect_in_keyword = true;
                            }
                            "in" | "function" | "repeat" => {
                                expect_command = false;
                            }
                            // fi, done, esac — block terminators
                            _ => {
                                expect_command = true;
                            }
                        }
                    } else {
                        tokens[i].token_type = TokenType::Word { command_position: true };
                        expect_command = false;
                    }
                }
                // If not expect_command, word stays as Word { command_position: false }
            }
            TokenType::Operator => {
                let text = &input[tokens[i].start..tokens[i].end];
                match text {
                    "{" | "}" => {
                        tokens[i].token_type = TokenType::ReservedWord;
                        if text == "{" {
                            expect_command = true;
                        }
                    }
                    "[[" | "]]" => {
                        tokens[i].token_type = TokenType::ReservedWord;
                        if text == "[[" {
                            expect_command = false;
                        }
                    }
                    "|" | "||" | "|&" | "&&" | ";" | ";;" | "(" => {
                        expect_command = true;
                    }
                    "&" => {
                        expect_command = true;
                    }
                    _ => {
                        // Redirects (>, >>, <, <<, <<<, )) don't change expect_command
                    }
                }
            }
            TokenType::String | TokenType::Comment | TokenType::ReservedWord => {
                if expect_command {
                    expect_command = false;
                }
            }
        }
    }
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

    // === Command position detection (US-003) ===

    fn cmd_word(start: usize, end: usize) -> Token {
        Token { start, end, token_type: TokenType::Word { command_position: true } }
    }

    fn reserved(start: usize, end: usize) -> Token {
        Token { start, end, token_type: TokenType::ReservedWord }
    }

    fn tokenize_with_positions(input: &str) -> Vec<Token> {
        let mut tokens = tokenize(input);
        mark_command_positions(&mut tokens, input);
        tokens
    }

    #[test]
    fn test_first_word_is_command_position() {
        // 'git status' — git is cmd, status is not
        assert_eq!(
            tokenize_with_positions("git status"),
            vec![cmd_word(0, 3), word(4, 10)]
        );
    }

    #[test]
    fn test_after_pipe_is_command_position() {
        // ls | grep foo — both ls and grep are command position
        assert_eq!(
            tokenize_with_positions("ls | grep foo"),
            vec![cmd_word(0, 2), operator(3, 4), cmd_word(5, 9), word(10, 13)]
        );
    }

    #[test]
    fn test_after_double_pipe_is_command_position() {
        assert_eq!(
            tokenize_with_positions("false || echo hi"),
            vec![cmd_word(0, 5), operator(6, 8), cmd_word(9, 13), word(14, 16)]
        );
    }

    #[test]
    fn test_after_double_ampersand_is_command_position() {
        assert_eq!(
            tokenize_with_positions("true && ls"),
            vec![cmd_word(0, 4), operator(5, 7), cmd_word(8, 10)]
        );
    }

    #[test]
    fn test_after_semicolon_is_command_position() {
        assert_eq!(
            tokenize_with_positions("cd /tmp; ls"),
            vec![cmd_word(0, 2), word(3, 7), operator(7, 8), cmd_word(9, 11)]
        );
    }

    #[test]
    fn test_after_double_semicolon_is_command_position() {
        assert_eq!(
            tokenize_with_positions("a;; b"),
            vec![cmd_word(0, 1), operator(1, 3), cmd_word(4, 5)]
        );
    }

    #[test]
    fn test_after_open_paren_is_command_position() {
        // (echo hi) — echo is command position
        assert_eq!(
            tokenize_with_positions("(echo hi)"),
            vec![operator(0, 1), cmd_word(1, 5), word(6, 8), operator(8, 9)]
        );
    }

    #[test]
    fn test_after_open_brace_is_command_position() {
        // { echo hi; } — brace is restyled, echo is command position
        assert_eq!(
            tokenize_with_positions("{ echo hi; }"),
            vec![
                reserved(0, 1),   // {
                cmd_word(2, 6),   // echo
                word(7, 9),       // hi
                operator(9, 10),  // ;
                reserved(11, 12), // }
            ]
        );
    }

    #[test]
    fn test_reserved_words_detected() {
        // if true; then echo hi; fi
        assert_eq!(
            tokenize_with_positions("if true; then echo hi; fi"),
            vec![
                reserved(0, 2),   // if
                cmd_word(3, 7),   // true
                operator(7, 8),   // ;
                reserved(9, 13),  // then
                cmd_word(14, 18), // echo
                word(19, 21),     // hi
                operator(21, 22), // ;
                reserved(23, 25), // fi
            ]
        );
    }

    #[test]
    fn test_after_then_is_command_position() {
        assert_eq!(
            tokenize_with_positions("then echo"),
            vec![reserved(0, 4), cmd_word(5, 9)]
        );
    }

    #[test]
    fn test_after_else_is_command_position() {
        assert_eq!(
            tokenize_with_positions("else echo"),
            vec![reserved(0, 4), cmd_word(5, 9)]
        );
    }

    #[test]
    fn test_after_elif_is_command_position() {
        assert_eq!(
            tokenize_with_positions("elif true"),
            vec![reserved(0, 4), cmd_word(5, 9)]
        );
    }

    #[test]
    fn test_after_do_is_command_position() {
        assert_eq!(
            tokenize_with_positions("do echo"),
            vec![reserved(0, 2), cmd_word(3, 7)]
        );
    }

    #[test]
    fn test_after_time_is_command_position() {
        assert_eq!(
            tokenize_with_positions("time ls"),
            vec![reserved(0, 4), cmd_word(5, 7)]
        );
    }

    #[test]
    fn test_bang_is_reserved_and_next_is_command() {
        // ! false — ! is reserved, false is command
        assert_eq!(
            tokenize_with_positions("! false"),
            vec![reserved(0, 1), cmd_word(2, 7)]
        );
    }

    #[test]
    fn test_after_for_not_command_position() {
        // for i in ... — i is NOT command position
        assert_eq!(
            tokenize_with_positions("for i in 1 2 3"),
            vec![
                reserved(0, 3), // for
                word(4, 5),     // i (not command position)
                reserved(6, 8), // in
                word(9, 10),    // 1 (not command position)
                word(11, 12),   // 2
                word(13, 14),   // 3
            ]
        );
    }

    #[test]
    fn test_after_case_not_command_position() {
        assert_eq!(
            tokenize_with_positions("case x"),
            vec![reserved(0, 4), word(5, 6)]
        );
    }

    #[test]
    fn test_after_select_not_command_position() {
        assert_eq!(
            tokenize_with_positions("select opt"),
            vec![reserved(0, 6), word(7, 10)]
        );
    }

    #[test]
    fn test_after_in_not_command_position() {
        assert_eq!(
            tokenize_with_positions("in a b c"),
            vec![reserved(0, 2), word(3, 4), word(5, 6), word(7, 8)]
        );
    }

    #[test]
    fn test_braces_restyled_to_reserved() {
        let tokens = tokenize_with_positions("{ }");
        assert_eq!(tokens[0].token_type, TokenType::ReservedWord);
        assert_eq!(tokens[1].token_type, TokenType::ReservedWord);
    }

    #[test]
    fn test_double_brackets_restyled_to_reserved() {
        let tokens = tokenize_with_positions("[[ -f file ]]");
        assert_eq!(tokens[0].token_type, TokenType::ReservedWord); // [[
        assert_eq!(tokens[3].token_type, TokenType::ReservedWord); // ]]
    }

    #[test]
    fn test_after_double_bracket_not_command_position() {
        // [[ -f file ]] — -f and file are NOT command position
        let tokens = tokenize_with_positions("[[ -f file ]]");
        assert_eq!(tokens[1], word(3, 5));   // -f (not cmd)
        assert_eq!(tokens[2], word(6, 10));  // file (not cmd)
    }

    #[test]
    fn test_second_word_not_command_position() {
        // echo hello — hello is not in command position
        assert_eq!(
            tokenize_with_positions("echo hello"),
            vec![cmd_word(0, 4), word(5, 10)]
        );
    }

    #[test]
    fn test_for_do_loop() {
        // for i in 1 2 3; do echo $i; done
        assert_eq!(
            tokenize_with_positions("for i in 1 2 3; do echo $i; done"),
            vec![
                reserved(0, 3),   // for
                word(4, 5),       // i
                reserved(6, 8),   // in
                word(9, 10),      // 1
                word(11, 12),     // 2
                word(13, 14),     // 3
                operator(14, 15), // ;
                reserved(16, 18), // do
                cmd_word(19, 23), // echo
                word(24, 26),     // $i
                operator(26, 27), // ;
                reserved(28, 32), // done
            ]
        );
    }

    #[test]
    fn test_while_loop() {
        assert_eq!(
            tokenize_with_positions("while true; do echo ok; done"),
            vec![
                reserved(0, 5),   // while
                cmd_word(6, 10),  // true
                operator(10, 11), // ;
                reserved(12, 14), // do
                cmd_word(15, 19), // echo
                word(20, 22),     // ok
                operator(22, 23), // ;
                reserved(24, 28), // done
            ]
        );
    }

    #[test]
    fn test_pipe_ampersand_command_position() {
        assert_eq!(
            tokenize_with_positions("a |& b"),
            vec![cmd_word(0, 1), operator(2, 4), cmd_word(5, 6)]
        );
    }

    #[test]
    fn test_background_ampersand_command_position() {
        // sleep 10 & ls — after &, ls is command position
        assert_eq!(
            tokenize_with_positions("sleep 10 & ls"),
            vec![cmd_word(0, 5), word(6, 8), operator(9, 10), cmd_word(11, 13)]
        );
    }
}
