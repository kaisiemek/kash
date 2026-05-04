mod builtins;
mod shell;

use std::io::BufReader;

use crate::shell::Shell;

fn main() {
    Shell::new(BufReader::new(std::io::stdin()), &mut std::io::stdout())
        .run_repl()
        .unwrap();
}
