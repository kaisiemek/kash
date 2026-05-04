use std::io::Write;

use crate::builtins::{is_builtin, run_builtin};

pub fn eval<W: Write>(writer: &mut W, argv: &[&str]) -> std::io::Result<()> {
    let Some(command) = argv.first() else {
        return Ok(());
    };

    if is_builtin(command) {
        run_builtin(writer, command, &argv[1..])
    } else {
        writeln!(writer, "{}: command not found", command)
    }
}
