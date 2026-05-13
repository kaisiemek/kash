use std::{
    fs::File,
    io::{BufRead, Read, Write},
    process::{Command, Stdio},
};

use crate::{builtins, errors::ShellError, parser::ShellCommand, shell::Shell};

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn run_command(&mut self, cmd: ShellCommand) {
        let Some(command) = cmd.argv.first() else {
            return;
        };

        let ret = if self.builtins.contains(&command.as_str()) {
            self.run_builtin(&cmd)
        } else if self.externals.contains_key(command) {
            self.run_external(&cmd)
        } else {
            Err(ShellError::UnknownCommand)
        };

        if let Err(err) = ret {
            self.report_error(err.wrap_in_execution_err(command.to_owned()));
        }
    }

    fn run_builtin(&mut self, cmd: &ShellCommand) -> Result<(), ShellError> {
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

        match Self::make_redirect_file(cmd)? {
            Some(mut file) => write!(file, "{}", output)?,
            None => write!(self.writer, "{}", output)?,
        }

        Ok(())
    }

    fn run_external(&mut self, cmd: &ShellCommand) -> Result<(), ShellError> {
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
            return Err(ShellError::PipeError);
        };
        let Some(mut stderr) = child.stderr.take() else {
            return Err(ShellError::PipeError);
        };

        let mut stderr_output = String::new();
        stderr.read_to_string(&mut stderr_output)?;
        write!(self.writer, "{}", stderr_output)?;

        let mut output = String::new();
        stdout.read_to_string(&mut output)?;
        match Self::make_redirect_file(cmd)? {
            Some(mut file) => write!(file, "{}", output)?,
            None => write!(self.writer, "{}", output)?,
        }
        Ok(())
    }

    fn make_redirect_file(cmd: &ShellCommand) -> Result<Option<File>, ShellError> {
        match &cmd.stdout_redirect {
            Some(path) => match File::create(path) {
                Ok(file) => Ok(Some(file)),
                Err(err) => Err(match err.kind() {
                    std::io::ErrorKind::NotFound => ShellError::NoSuchFileOrDir(path.to_owned()),
                    std::io::ErrorKind::PermissionDenied => {
                        ShellError::PermissionDenied(path.to_owned())
                    }
                    _ => err.into(),
                }),
            },
            None => Ok(None),
        }
    }
}
