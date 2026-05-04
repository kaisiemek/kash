mod builtins;
mod eval;
mod repl;

use std::io::BufReader;

use crate::repl::Repl;

fn main() {
    Repl::new(BufReader::new(std::io::stdin()), std::io::stdout())
        .run()
        .unwrap();
}
