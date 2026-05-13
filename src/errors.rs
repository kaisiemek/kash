use std::{
    io::{BufRead, Write},
    path::PathBuf,
};

use crate::shell::Shell;

pub enum ShellError {
    NoSuchFileOrDir(PathBuf),
    PermissionDenied(PathBuf),
    UnknownCommand,
    PipeError,
    IoError(std::io::Error),
}
pub struct ExecutionErr {
    command: String,
    errtype: ShellError,
}

#[derive(Debug)]
pub enum ParserError {
    UnterminatedSingleQuoteString,
    UnterminatedDoubleQuoteString,
    NeedNextLine,
    UnexpectedEnd,
    UnexpectedToken,
}

impl From<std::io::Error> for ShellError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl ShellError {
    pub fn wrap_in_execution_err(self, command: String) -> ExecutionErr {
        ExecutionErr {
            command,
            errtype: self,
        }
    }
}

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn report_error(&mut self, err: ExecutionErr) {
        match err.errtype {
            ShellError::NoSuchFileOrDir(path) => writeln!(
                self.writer,
                "{}: {}: No such file or directory",
                err.command,
                path.display()
            ),
            ShellError::PermissionDenied(path) => writeln!(
                self.writer,
                "{}: {}: Permission denied",
                err.command,
                path.display()
            ),
            ShellError::UnknownCommand => {
                writeln!(self.writer, "{}: command not found", err.command)
            }
            ShellError::PipeError => writeln!(self.writer, "{}: couldn't get pipe", err.command),
            ShellError::IoError(error) => {
                writeln!(self.writer, "{}: IO Error: {}", err.command, error)
            }
        }
        .expect("couldn't report error, aborting...");
    }
}
