mod tokenizer;

use std::{path::PathBuf, vec::IntoIter};

use crate::parser::tokenizer::{Token, Tokenizer};

#[derive(Debug)]
pub enum ParserError {
    UnterminatedSingleQuoteString,
    UnterminatedDoubleQuoteString,
    NeedNextLine,
    UnexpectedEnd,
    UnexpectedToken,
}

pub struct CommandParser {
    tokenizer: Tokenizer,
    tokens: IntoIter<Token>,
}

#[derive(Debug, Default)]
pub struct ShellCommand {
    pub argv: Vec<String>,
    pub stdout_redirect: Option<PathBuf>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            tokenizer: Tokenizer::new(),
            tokens: Vec::new().into_iter(),
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<ShellCommand, ParserError> {
        self.tokens = self.tokenizer.tokenize(input)?.into_iter();
        let mut command = ShellCommand::default();

        while let Some(token) = self.tokens.next() {
            match token {
                Token::Word(word) => command.argv.push(word),
                Token::StdoutRedirect => {
                    command.stdout_redirect = Some(PathBuf::from(self.expect_word()?))
                }
            }
        }

        Ok(command)
    }

    fn expect_word(&mut self) -> Result<String, ParserError> {
        match self.tokens.next() {
            None => Err(ParserError::UnexpectedEnd),
            Some(Token::Word(word)) => Ok(word),
            _ => Err(ParserError::UnexpectedToken),
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::parser::CommandParser;

    fn run_tests(inputs: Vec<&str>, expected_outputs: Vec<Vec<&str>>) {
        let expected_outputs: Vec<Vec<String>> = expected_outputs
            .iter()
            .map(|strvec| strvec.iter().map(|s| s.to_string()).collect())
            .collect();

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            let mut parser = CommandParser::new();
            let mut input_line = input.to_string();
            input_line.push('\n');
            let cmd = parser.parse(&input_line).unwrap();
            assert_eq!(cmd.argv, expected_output, "input: {}", input);
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

    #[test]
    fn test_redirects() {
        let inputs = vec![
            "echo 'abc' > testfile",
            "echo 'abc' > testfile1 > testfile2",
            "echo 'abc' > subdir/testfile",
        ];
        let expected_outputs = vec!["testfile", "testfile2", "subdir/testfile"];
        let expected_outputs: Vec<PathBuf> =
            expected_outputs.iter().map(|s| PathBuf::from(s)).collect();

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            let mut parser = CommandParser::new();
            let mut input_line = input.to_string();
            input_line.push('\n');
            let cmd = parser.parse(&input_line).unwrap();
            assert_eq!(
                cmd.stdout_redirect.unwrap(),
                expected_output,
                "input: {}",
                input
            );
        }
    }
}
