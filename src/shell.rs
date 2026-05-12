use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::PathBuf,
};

use crate::{
    builtins,
    parser::{CommandParser, ParserError},
};

pub struct Shell<R: BufRead, W: Write> {
    pub(crate) reader: R,
    pub(crate) writer: W,
    pub(crate) externals: HashMap<String, PathBuf>,
    pub(crate) builtins: Vec<&'static str>,
    prompt: &'static str,
    parser: CommandParser,
    buf: String,
}

impl<R: BufRead, W: Write> Shell<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            externals: Self::collect_externals(),
            builtins: builtins::get_builtins(),
            prompt: "$",
            parser: CommandParser::new(),
            buf: String::new(),
        }
    }

    pub fn run_repl(&mut self) -> std::io::Result<()> {
        loop {
            write!(self.writer, "{} ", self.prompt)?;
            self.writer.flush()?;

            let bytesread = self.reader.read_line(&mut self.buf)?;
            if bytesread == 0 {
                break;
            }

            self.eval_line()?;
        }
        Ok(())
    }

    pub fn eval_line(&mut self) -> std::io::Result<()> {
        let cmd = match self.parser.parse(&self.buf) {
            Ok(cmd) => cmd,
            Err(err) => {
                return self.handle_parser_err(err);
            }
        };
        // clear the linebuf only when the parsing was successful
        // (or if the error can't be resolved by keeping parsing further lines)
        self.buf.clear();
        self.prompt = "$";

        if let Err(err) = self.run_command(cmd) {
            writeln!(self.writer, "{}", err)?;
        }

        // if self.builtins.contains(&cmd_name.as_str()) {
        //     // TODO: move error handling in the run_builtin function
        //
        //     if let Err(err) = self.run_builtin(cmd_name, &cmd.argv[1..]) {
        //         writeln!(self.writer, "{}: {}", cmd_name, err)?;
        //     }
        // } else if self.externals.contains_key(cmd_name) {
        //     self.run_external(cmd_name, &cmd.argv[1..])?;
        // } else {
        //     writeln!(self.writer, "{}: command not found", cmd_name)?;
        // }
        //
        Ok(())
    }

    fn handle_parser_err(&mut self, err: ParserError) -> std::io::Result<()> {
        match err {
            ParserError::UnterminatedSingleQuoteString => self.prompt = "quote>",
            ParserError::UnterminatedDoubleQuoteString => self.prompt = "dquote>",
            ParserError::NeedNextLine => self.prompt = ">",
            ParserError::UnexpectedEnd => {
                writeln!(self.writer, "parse error: unexpected end")?;
                self.buf.clear();
            }
            ParserError::UnexpectedToken => {
                writeln!(self.writer, "parse error: unexpected token")?;
                self.buf.clear();
            }
        };
        Ok(())
    }
}
