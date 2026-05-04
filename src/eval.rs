use std::io::Write;

use crate::builtins::{is_builtin, run_builtin};

pub fn eval<W: Write>(writer: &mut W, input: &str) -> std::io::Result<()> {
    let argv = parse_argv(input);

    let Some(command) = argv.first() else {
        return Ok(());
    };

    if is_builtin(command) {
        run_builtin(writer, command, &argv[1..])
    } else {
        writeln!(writer, "{}: command not found", command)
    }
}

fn parse_argv(line: &str) -> Vec<&str> {
    line.trim().split_ascii_whitespace().collect()
}
