use std::io::{BufRead, Write};

use crate::eval::eval;

pub struct Repl<R: BufRead, W: Write> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> Repl<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut input = String::new();
        loop {
            write!(self.writer, "$ ")?;
            self.writer.flush()?;

            let bytesread = self.reader.read_line(&mut input)?;
            if bytesread == 0 {
                break;
            }

            let argv = Self::parse_argv(&input);
            eval(&mut self.writer, &argv)?;
            input.clear();
        }

        Ok(())
    }

    fn parse_argv(line: &str) -> Vec<&str> {
        line.trim().split_ascii_whitespace().collect()
    }
}
