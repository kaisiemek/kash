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
                    self.parse_single_quoted();
                }
                '"' => {
                    self.iterator.next();
                    self.parse_double_quoted();
                }
                '\\' => {
                    self.iterator.next();
                    self.escape_next();
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

    fn parse_single_quoted(&mut self) {
        while let Some(c) = self.iterator.next() {
            match c {
                '\'' => break,
                _ => self.current_arg.push(c),
            }
        }
    }

    fn parse_double_quoted(&mut self) {
        while let Some(c) = self.iterator.next() {
            match c {
                '"' => break,
                '\\' => {
                    let Some(cn) = self.iterator.peek() else {
                        self.current_arg.push('\\');
                        break;
                    };
                    // only escape ", \, $, `, \n while in double quotes
                    match cn {
                        '"' | '\\' | '$' | '`' | '\n' => self.escape_next(),
                        _ => self.current_arg.push('\\'),
                    }
                }

                _ => self.current_arg.push(c),
            }
        }
    }

    fn escape_next(&mut self) {
        if let Some(c) = self.iterator.next() {
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

    fn run_tests(inputs: Vec<&str>, expected_outputs: Vec<Vec<&str>>) {
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

    #[test]
    fn test_spaces() {
        let inputs = vec![
            "echo abc def hij",
            "echo        abc          def        hij     ",
            "      echo abc def",
        ];

        let expected_outputs = vec![
            vec!["echo", "abc", "def", "hij"],
            vec!["echo", "abc", "def", "hij"],
            vec!["echo", "abc", "def"],
        ];

        run_tests(inputs, expected_outputs);
    }

    #[test]
    fn test_quoting() {
        let inputs = vec![
            "echo 'abc  def'  ",
            "echo 'abc \"def\"'' hij'",
            "echo 'abc def' 'hij  '",
            "echo '' abc' d e f'",
            "echo abc' d e f' ''",
            "echo abc\" d e f' ''\"",
            "echo \"abc def\"\"hi\" \"j  \"",
        ];

        let expected_outputs = vec![
            vec!["echo", "abc  def"],
            vec!["echo", "abc \"def\" hij"],
            vec!["echo", "abc def", "hij  "],
            vec!["echo", "abc d e f"],
            vec!["echo", "abc d e f"],
            vec!["echo", "abc d e f' ''"],
            vec!["echo", "abc defhi", "j  "],
        ];

        run_tests(inputs, expected_outputs);
    }

    #[test]
    fn test_escaping() {
        let inputs = vec![
            "echo a\\ b\\ c",
            "echo a\\     b",
            "echo a\\b\\c",
            "echo a\\\\b",
            "echo \\\"abc\\\"",
            "echo \\'abc\\'",
            "echo ' a \\' ' b '",
            r##"echo " a \\\" b ""##,
            "echo \"a \\ \\b\"",
        ];

        let expected_outputs = vec![
            vec!["echo", "a b c"],
            vec!["echo", "a ", "b"],
            vec!["echo", "abc"],
            vec!["echo", "a\\b"],
            vec!["echo", "\"abc\""],
            vec!["echo", "'abc'"],
            vec!["echo", " a \\", " b "],
            vec!["echo", r##" a \" b "##],
            vec!["echo", "a \\ \\b"],
        ];

        run_tests(inputs, expected_outputs);
    }
}
