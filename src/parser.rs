use std::{iter::Peekable, str::Chars};

pub struct ArgvParser<'a> {
    iterator: Peekable<Chars<'a>>,
    argv: Vec<String>,
    current_arg: String,
}

impl<'a> ArgvParser<'a> {
    pub fn parse_argv(input: &'a str) -> Vec<String> {
        Self::new(input).get_argv()
    }

    fn new(input: &'a str) -> Self {
        Self {
            iterator: input.trim().chars().peekable(),
            argv: Vec::new(),
            current_arg: String::new(),
        }
    }

    fn get_argv(mut self) -> Vec<String> {
        while let Some(c) = self.iterator.peek() {
            match c {
                ' ' => {
                    self.add_arg();
                    self.skip_whitespace();
                }
                '\'' => {
                    self.iterator.next();
                    self.parse_quoted();
                }
                _ => {
                    self.current_arg.push(*c);
                    self.iterator.next();
                }
            }
        }
        self.add_arg();

        self.argv
    }

    fn skip_whitespace(&mut self) {
        while self.iterator.peek().is_some_and(|c| c.is_whitespace()) {
            self.iterator.next();
        }
    }

    fn parse_quoted(&mut self) {
        while let Some(c) = self.iterator.next()
            && c != '\''
        {
            self.current_arg.push(c);
        }
    }

    fn add_arg(&mut self) {
        if !self.current_arg.is_empty() {
            self.argv.push(std::mem::take(&mut self.current_arg));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::ArgvParser;

    #[test]
    fn test_argv_parsing() {
        let inputs = vec![
            "echo abc def hij",
            "echo        abc          def        hij     ",
            "echo 'abc  def'  ",
            "echo 'abc def'' hij'",
            "echo 'abc def' 'hij  '",
            "echo '' abc' d e f'",
            "echo abc' d e f' ''",
        ];

        let expected_outputs = vec![
            vec!["echo", "abc", "def", "hij"],
            vec!["echo", "abc", "def", "hij"],
            vec!["echo", "abc  def"],
            vec!["echo", "abc def hij"],
            vec!["echo", "abc def", "hij  "],
            vec!["echo", "abc d e f"],
        ];

        let expected_outputs: Vec<Vec<String>> = expected_outputs
            .iter()
            .map(|strvec| strvec.iter().map(|s| s.to_string()).collect())
            .collect();

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            assert_eq!(
                ArgvParser::parse_argv(input),
                expected_output,
                "input: {}",
                input
            );
        }
    }
}
