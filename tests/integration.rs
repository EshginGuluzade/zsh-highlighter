use std::collections::HashSet;

use zsh_highlighter::classifier::{classify, Highlight};
use zsh_highlighter::tokenizer::{mark_command_positions, tokenize};

/// Helper: run the full tokenize+classify pipeline and return highlights.
fn highlight(input: &str, known: &[&str]) -> Vec<Highlight> {
    let commands: HashSet<&str> = known.iter().copied().collect();
    let mut tokens = tokenize(input);
    mark_command_positions(&mut tokens, input);
    classify(&tokens, input, &commands)
}

#[test]
fn test_ls_with_args() {
    // ls gets fg=green,bold; -la and /tmp get no highlight
    let result = highlight("ls -la /tmp", &["ls"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 2, style: "fg=green,bold" },
    ]);
}

#[test]
fn test_invalid_command() {
    // gti is unknown — gets fg=red,underline
    let result = highlight("gti status", &["git"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 3, style: "fg=red,underline" },
    ]);
}

#[test]
fn test_echo_double_quoted_string() {
    // echo gets fg=green,bold, the quoted string gets fg=yellow
    let result = highlight("echo \"hello world\"", &["echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 4, style: "fg=green,bold" },
        Highlight { start: 5, end: 18, style: "fg=yellow" },
    ]);
}

#[test]
fn test_pipe() {
    // cat file.txt | grep pattern — both commands fg=green,bold, pipe fg=cyan
    let result = highlight("cat file.txt | grep pattern", &["cat", "grep"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 3, style: "fg=green,bold" },
        Highlight { start: 13, end: 14, style: "fg=cyan" },
        Highlight { start: 15, end: 19, style: "fg=green,bold" },
    ]);
}

#[test]
fn test_if_then_fi() {
    // if/then/fi fg=yellow,bold, true/echo fg=green,bold, semicolons fg=cyan
    let result = highlight("if true; then echo ok; fi", &["true", "echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 2, style: "fg=yellow,bold" },    // if
        Highlight { start: 3, end: 7, style: "fg=green,bold" },     // true
        Highlight { start: 7, end: 8, style: "fg=cyan" },           // ;
        Highlight { start: 9, end: 13, style: "fg=yellow,bold" },   // then
        Highlight { start: 14, end: 18, style: "fg=green,bold" },   // echo
        Highlight { start: 21, end: 22, style: "fg=cyan" },         // ;
        Highlight { start: 23, end: 25, style: "fg=yellow,bold" },  // fi
    ]);
}

#[test]
fn test_comment_only() {
    // entire input fg=8
    let result = highlight("# this is a comment", &[]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 19, style: "fg=8" },
    ]);
}

#[test]
fn test_inline_comment() {
    // echo fg=green,bold, hello unstyled, '# inline comment' fg=8
    let result = highlight("echo hello # inline comment", &["echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 4, style: "fg=green,bold" },
        Highlight { start: 11, end: 27, style: "fg=8" },
    ]);
}

#[test]
fn test_empty_string() {
    let result = highlight("", &[]);
    assert_eq!(result, Vec::new());
}

#[test]
fn test_for_in_do_done() {
    // for/in/do/done fg=yellow,bold, echo fg=green,bold
    let result = highlight("for i in 1 2 3; do echo $i; done", &["echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 3, style: "fg=yellow,bold" },    // for
        Highlight { start: 6, end: 8, style: "fg=yellow,bold" },    // in
        Highlight { start: 14, end: 15, style: "fg=cyan" },         // ;
        Highlight { start: 16, end: 18, style: "fg=yellow,bold" },  // do
        Highlight { start: 19, end: 23, style: "fg=green,bold" },   // echo
        Highlight { start: 26, end: 27, style: "fg=cyan" },         // ;
        Highlight { start: 28, end: 32, style: "fg=yellow,bold" },  // done
    ]);
}

#[test]
fn test_ansi_c_string() {
    // string portion gets fg=yellow
    let result = highlight("echo $'hello\\nworld'", &["echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 4, style: "fg=green,bold" },
        Highlight { start: 5, end: 20, style: "fg=yellow" },
    ]);
}

#[test]
fn test_redirect() {
    // redirect > gets fg=cyan
    let result = highlight("echo hello > file", &["echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 4, style: "fg=green,bold" },
        Highlight { start: 11, end: 12, style: "fg=cyan" },
    ]);
}

#[test]
fn test_braces() {
    // braces fg=yellow,bold (reserved word), echo fg=green,bold, semicolons fg=cyan
    let result = highlight("{ echo hi; }", &["echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 1, style: "fg=yellow,bold" },    // {
        Highlight { start: 2, end: 6, style: "fg=green,bold" },     // echo
        Highlight { start: 9, end: 10, style: "fg=cyan" },          // ;
        Highlight { start: 11, end: 12, style: "fg=yellow,bold" },  // }
    ]);
}

#[test]
fn test_double_brackets() {
    // [[ and ]] get fg=yellow,bold
    let result = highlight("[[ -f file ]]", &[]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 2, style: "fg=yellow,bold" },    // [[
        Highlight { start: 11, end: 13, style: "fg=yellow,bold" },  // ]]
    ]);
}

#[test]
fn test_logical_operators() {
    // all three commands fg=green,bold, && and || fg=cyan
    let result = highlight("true && false || echo done", &["true", "false", "echo"]);
    assert_eq!(result, vec![
        Highlight { start: 0, end: 4, style: "fg=green,bold" },     // true
        Highlight { start: 5, end: 7, style: "fg=cyan" },           // &&
        Highlight { start: 8, end: 13, style: "fg=green,bold" },    // false
        Highlight { start: 14, end: 16, style: "fg=cyan" },         // ||
        Highlight { start: 17, end: 21, style: "fg=green,bold" },   // echo
    ]);
}
