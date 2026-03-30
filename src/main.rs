mod classifier;
mod tokenizer;

use std::collections::HashSet;
use std::env;

fn main() {
    let args: Vec<std::string::String> = env::args().collect();

    let input = match args.get(1) {
        Some(s) if !s.is_empty() => s.as_str(),
        _ => return,
    };

    let cmds_env = env::var("_ZH_CMDS").unwrap_or_default();
    let known_commands: HashSet<&str> = if cmds_env.is_empty() {
        HashSet::new()
    } else {
        cmds_env.split('\n').filter(|s| !s.is_empty()).collect()
    };

    let tokens = tokenizer::tokenize(input);
    let highlights = classifier::classify(&tokens, &known_commands);

    for h in &highlights {
        println!("{} {} {}", h.start, h.end, h.style);
    }
}
