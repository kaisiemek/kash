use std::io::{BufRead, Write};

use crate::shell::Shell;

impl<'a, R: BufRead, W: Write> Shell<'a, R, W> {
    pub fn get_builtins() -> Vec<&'static str> {
        vec!["echo", "exit", "type"]
    }

    pub fn run_builtin(&mut self, command: &str, argv: &[&str]) -> std::io::Result<()> {
        match command {
            "echo" => self.echo(argv),
            "exit" => Self::exit(),
            "type" => self.typebuiltin(argv),
            _ => Ok(()),
        }
    }

    fn echo(&mut self, argv: &[&str]) -> std::io::Result<()> {
        writeln!(self.writer, "{}", argv.join(" "))
    }

    fn exit() -> std::io::Result<()> {
        std::process::exit(0);
    }

    fn typebuiltin(&mut self, argv: &[&str]) -> std::io::Result<()> {
        for arg in argv {
            if self.builtins.contains(arg) {
                writeln!(self.writer, "{} is a shell builtin", arg)?;
            } else {
                writeln!(self.writer, "{} not found", arg)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Cursor};

    use crate::shell::Shell;

    #[test]
    fn test_echo() {
        let inputs = vec![
            "echo",
            "echo abc",
            "echo abc def",
            "     echo abc",
            "echo abc  ",
            " echo  abc  def  ",
        ];
        let expected_results = vec!["\n", "abc\n", "abc def\n", "abc\n", "abc\n", "abc def\n"];

        let mut output = Vec::new();
        for (input, expected) in inputs.into_iter().zip(expected_results) {
            let mut shell = Shell::new(BufReader::new(Cursor::new("")), &mut output);
            shell.eval_line(input).unwrap();
            assert_eq!(String::from_utf8_lossy(&output), expected);
            output.clear();
        }
    }

    #[test]
    fn test_type() {
        let inputs = vec![
            "type echo",
            "type echo type exit",
            "type echo notexist   ",
            "type type type",
            "  type     notexist  ",
        ];
        let expected_results = vec![
            "echo is a shell builtin\n",
            "echo is a shell builtin\ntype is a shell builtin\nexit is a shell builtin\n",
            "echo is a shell builtin\nnotexist not found\n",
            "type is a shell builtin\ntype is a shell builtin\n",
            "notexist not found\n",
        ];

        let mut output = Vec::new();
        for (input, expected) in inputs.into_iter().zip(expected_results) {
            let mut shell = Shell::new(BufReader::new(Cursor::new("")), &mut output);
            shell.eval_line(input).unwrap();
            assert_eq!(String::from_utf8_lossy(&output), expected);
            output.clear();
        }
    }
}
