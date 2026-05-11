use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::PathBuf,
};

use crate::parser::{ParserError, parse_argv};

pub struct Shell<R: BufRead, W: Write> {
    pub(crate) reader: R,
    pub(crate) writer: W,
    pub(crate) externals: HashMap<String, PathBuf>,
    pub(crate) builtins: Vec<&'static str>,
    prompt: &'static str,
}

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            externals: Self::collect_externals(),
            builtins: Self::get_builtins(),
            prompt: "$",
        }
    }

    pub fn run_repl(&mut self) -> std::io::Result<()> {
        let mut buf = String::new();
        loop {
            write!(self.writer, "{} ", self.prompt)?;
            self.writer.flush()?;

            let bytesread = self.reader.read_line(&mut buf)?;
            if bytesread == 0 {
                break;
            }

            // only clear the buffer if the line has been evaluated successfully
            if self.eval_line(&buf)? {
                buf.clear();
            }
        }
        Ok(())
    }

    pub fn eval_line(&mut self, line: &str) -> std::io::Result<bool> {
        let argv = match parse_argv(line) {
            Ok(argv) => argv,
            Err(err) => {
                self.set_prompt(Some(err));
                return Ok(false);
            }
        };
        self.set_prompt(None);

        // just return and wait for next command for empty argvs
        let Some(command) = argv.first() else {
            return Ok(true);
        };

        if self.builtins.contains(&command.as_str()) {
            // TODO: move error handling in the run_builtin function
            if let Err(err) = self.run_builtin(command, &argv[1..]) {
                writeln!(self.writer, "{}: {}", command, err)?;
            }
        } else if self.externals.contains_key(command) {
            self.run_external(command, &argv[1..])?;
        } else {
            writeln!(self.writer, "{}: command not found", command)?;
        }

        Ok(true)
    }

    fn set_prompt(&mut self, err: Option<ParserError>) {
        self.prompt = match err {
            Some(ParserError::NeedNextLine) => ">",
            Some(ParserError::UnterminatedSingleQuoteString) => "quote>",
            Some(ParserError::UnterminatedDoubleQuoteString) => "dquote>",
            None => "$",
        };
    }
}
