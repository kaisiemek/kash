use std::{
    fs::File,
    io::{BufRead, Read, Write},
    process::{Command, Stdio},
};

use crate::{builtins, parser::ShellCommand, shell::Shell};

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn run_command(&mut self, cmd: ShellCommand) -> std::io::Result<()> {
        let Some(command) = cmd.argv.first() else {
            return Ok(());
        };

        if self.builtins.contains(&command.as_str()) {
            self.run_builtin(cmd)
        } else if self.externals.contains_key(command) {
            self.run_external(cmd)
        } else {
            writeln!(self.writer, "{}: command not found", command)
        }
    }

    fn run_builtin(&mut self, cmd: ShellCommand) -> std::io::Result<()> {
        let Some(builtin) = cmd.argv.first() else {
            return Ok(());
        };

        let output = match builtin.as_str() {
            "cd" => builtins::cd(&cmd.argv[1..]),
            "echo" => builtins::echo(&cmd.argv[1..]),
            "exit" => builtins::exit(),
            "pwd" => builtins::pwd(),
            "type" => builtins::typebuiltin(&cmd.argv[1..], &self.builtins, &self.externals),
            _ => "".to_string(),
        };

        if let Some(path) = cmd.stdout_redirect {
            let mut file = File::create(path)?;
            write!(file, "{}", output)
        } else {
            write!(self.writer, "{}", output)
        }
    }

    fn run_external(&mut self, cmd: ShellCommand) -> std::io::Result<()> {
        let Some(external) = cmd.argv.first() else {
            return Ok(());
        };
        let Some(_) = self.externals.get(external) else {
            return Ok(());
        };

        let mut child = Command::new(external)
            .args(&cmd.argv[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let Some(mut stdout) = child.stdout.take() else {
            return Ok(());
        };

        let mut output = String::new();
        stdout.read_to_string(&mut output)?;
        if let Some(path) = cmd.stdout_redirect {
            let mut file = File::create(path)?;
            write!(file, "{}", output)
        } else {
            write!(self.writer, "{}", output)
        }
    }
}
