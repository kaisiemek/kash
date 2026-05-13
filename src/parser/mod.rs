mod tokenizer;

use std::{path::PathBuf, vec::IntoIter};

use crate::{
    errors::ParserError,
    parser::tokenizer::{RedirectType, Token, Tokenizer},
};
pub struct CommandParser {
    tokenizer: Tokenizer,
    tokens: IntoIter<Token>,
}

#[derive(Debug, Default)]
pub struct ShellCommand {
    pub argv: Vec<String>,
    pub stdout_redirect: Option<PipeRedirect>,
    pub stderr_redirect: Option<PipeRedirect>,
}

#[derive(Debug, PartialEq)]
pub struct PipeRedirect {
    pub path: PathBuf,
    pub mode: RedirectMode,
}

#[derive(Debug, PartialEq)]
pub enum RedirectMode {
    Append,
    Overwrite,
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
                Token::RedirectOperator(red_type) => self.parse_redirect(&mut command, red_type)?,
            }
        }

        Ok(command)
    }

    fn parse_redirect(
        &mut self,
        command: &mut ShellCommand,
        red_type: RedirectType,
    ) -> Result<(), ParserError> {
        let redirect = Some(PipeRedirect {
            path: PathBuf::from(self.expect_word()?),
            mode: match red_type {
                RedirectType::Stdout | RedirectType::Stderr => RedirectMode::Overwrite,
                RedirectType::StdoutAppend | RedirectType::StderrAppend => RedirectMode::Append,
            },
        });
        match red_type {
            RedirectType::Stdout | RedirectType::StdoutAppend => command.stdout_redirect = redirect,
            RedirectType::Stderr | RedirectType::StderrAppend => command.stderr_redirect = redirect,
        }
        Ok(())
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
    use crate::parser::{CommandParser, PipeRedirect, RedirectMode};

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

    fn make_redirect(path: &str, mode: &str) -> Option<PipeRedirect> {
        let mode = if mode == "a" {
            RedirectMode::Append
        } else {
            RedirectMode::Overwrite
        };

        Some(PipeRedirect {
            path: path.into(),
            mode,
        })
    }

    #[test]
    fn test_redirects() {
        let inputs = vec![
            "echo 'abc'",
            "echo 'abc' > out",
            "echo 'abc' > out 1> out2",
            "echo 'abc' > subdir/out",
            "echo 'abc' 1> out 2> err",
            "echo 'abc' 1>> out 2>> err 2> err2",
            "echo 'abc' 2>> err >> out",
            "echo>>out 'abc'2>>err",
            "echo>'out file.txt' abc",
        ];
        let expected_outputs = vec![
            (None, None),
            (make_redirect("out", "o"), None),
            (make_redirect("out2", "o"), None),
            (make_redirect("subdir/out", "o"), None),
            (make_redirect("out", "o"), make_redirect("err", "o")),
            (make_redirect("out", "a"), make_redirect("err2", "o")),
            (make_redirect("out", "a"), make_redirect("err", "a")),
            (make_redirect("out", "a"), make_redirect("err", "a")),
            (make_redirect("out file.txt", "o"), None),
        ];

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            let mut parser = CommandParser::new();
            let mut input_line = input.to_string();
            input_line.push('\n');
            let cmd = parser.parse(&input_line).unwrap();
            assert_eq!(
                cmd.stdout_redirect, expected_output.0,
                "stdout, input: {}",
                input
            );
            assert_eq!(
                cmd.stderr_redirect, expected_output.1,
                "stderr, input: {}",
                input
            );
        }
    }
}
