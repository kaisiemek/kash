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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_argv() {
        let inputs = vec![
            "echo",
            "echo abc",
            "echo abc def",
            "     echo abc",
            "echo abc  ",
            " echo  abc  def  ",
        ];
        let expected_results = vec![
            ["echo"].as_slice(),
            ["echo", "abc"].as_slice(),
            ["echo", "abc", "def"].as_slice(),
            ["echo", "abc"].as_slice(),
            ["echo", "abc"].as_slice(),
            ["echo", "abc", "def"].as_slice(),
        ];

        for (input, expected) in inputs.into_iter().zip(expected_results) {
            assert_eq!(&parse_argv(input), expected);
        }
    }
}
