mod builtins;
mod externals;
mod parser;
mod shell;

use std::io::BufReader;

use crate::shell::Shell;

fn main() {
    Shell::new(BufReader::new(std::io::stdin()), std::io::stdout())
        .run_repl()
        .unwrap();
}
