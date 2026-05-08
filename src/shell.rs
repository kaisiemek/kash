use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::PathBuf,
};

use crate::parser::ArgvParser;

pub struct Shell<R: BufRead, W: Write> {
    pub(crate) reader: R,
    pub(crate) writer: W,
    pub(crate) externals: HashMap<String, PathBuf>,
    pub(crate) builtins: Vec<&'static str>,
}

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            externals: Self::collect_externals(),
            builtins: Self::get_builtins(),
        }
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
        let argv = ArgvParser::parse_argv(line);

        let Some(command) = argv.first() else {
            return Ok(());
        };

        if self.builtins.contains(&command.as_str()) {
            if let Err(err) = self.run_builtin(command, &argv[1..]) {
                writeln!(self.writer, "{}: {}", command, err)
            } else {
                Ok(())
            }
        } else if self.externals.contains_key(command) {
            self.run_external(command, &argv[1..])
        } else {
            writeln!(self.writer, "{}: command not found", command)
        }
    }
}
