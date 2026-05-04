use std::io::{BufRead, Write};

use crate::builtins::{is_builtin, run_builtin};

pub struct Shell<'a, R: BufRead, W: Write> {
    reader: R,
    writer: &'a mut W,
}

impl<'a, R: BufRead, W: Write> Shell<'a, R, W> {
    pub fn new(reader: R, writer: &'a mut W) -> Self {
        Self { reader, writer }
    }

    pub fn run_repl(&mut self) -> std::io::Result<()> {
        let mut buf = String::new();
        loop {
            write!(self.writer, "$ ")?;
            self.writer.flush()?;

            let bytesread = self.reader.read_line(&mut buf)?;
            if bytesread == 0 {
                break;
            }

            self.eval_line(&buf)?;
            buf.clear();
        }

        Ok(())
    }
    pub fn eval_line(&mut self, line: &str) -> std::io::Result<()> {
        let argv = parse_argv(line);

        let Some(command) = argv.first() else {
            return Ok(());
        };

        if is_builtin(command) {
            run_builtin(&mut self.writer, command, &argv[1..])
        } else {
            writeln!(self.writer, "{}: command not found", command)
        }
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
