use std::{
    fs::{File, OpenOptions},
    io::{BufRead, Read, Write},
    process::{Command, Stdio},
};

use crate::{
    builtins,
    errors::ShellError,
    parser::{PipeRedirect, RedirectMode, ShellCommand},
    shell::Shell,
};

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

        match Self::make_redirect_file(&cmd.stdout_redirect)? {
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

        self.write_pipe_output(&mut stdout, &cmd.stdout_redirect)?;
        self.write_pipe_output(&mut stderr, &cmd.stderr_redirect)?;
        Ok(())
    }

    fn write_pipe_output<R2: Read>(
        &mut self,
        pipe_output: &mut R2,
        redirect: &Option<PipeRedirect>,
    ) -> Result<(), ShellError> {
        let mut buf = String::new();
        pipe_output.read_to_string(&mut buf)?;
        match Self::make_redirect_file(redirect)? {
            Some(mut file) => write!(file, "{}", buf)?,
            None => write!(self.writer, "{}", buf)?,
        }
        Ok(())
    }

    fn make_redirect_file(redirect: &Option<PipeRedirect>) -> Result<Option<File>, ShellError> {
        let Some(redirect) = redirect else {
            return Ok(None);
        };
        let mut open_opts = OpenOptions::new();
        open_opts.create(true).write(true);
        match redirect.mode {
            RedirectMode::Append => open_opts.append(true),
            RedirectMode::Overwrite => open_opts.truncate(true),
        };
        match open_opts.open(&redirect.path) {
            Ok(file) => Ok(Some(file)),
            Err(err) => Err(match err.kind() {
                std::io::ErrorKind::NotFound => {
                    ShellError::NoSuchFileOrDir(redirect.path.to_owned())
                }
                std::io::ErrorKind::PermissionDenied => {
                    ShellError::PermissionDenied(redirect.path.to_owned())
                }
                _ => err.into(),
            }),
        }
    }
}
