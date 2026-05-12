mod tokenizer;

use crate::parser::tokenizer::{Token, Tokenizer};

#[derive(Debug)]
pub enum ParserError {
    UnterminatedSingleQuoteString,
    UnterminatedDoubleQuoteString,
    NeedNextLine,
}

pub fn parse_argv(input: &str) -> Result<Vec<String>, ParserError> {
    let mut argv = Vec::new();
    let tokens = Tokenizer::new().tokenize(input)?;
    for token in tokens {
        match token {
            Token::Word(word) => argv.push(word),
            Token::StdoutRedirect => unimplemented!(),
        }
    }
    Ok(argv)
}

#[cfg(test)]
mod test {
    use crate::parser::parse_argv;

    fn run_tests(inputs: Vec<&str>, expected_outputs: Vec<Vec<&str>>) {
        let expected_outputs: Vec<Vec<String>> = expected_outputs
            .iter()
            .map(|strvec| strvec.iter().map(|s| s.to_string()).collect())
            .collect();

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            let mut input_line = input.to_string();
            input_line.push('\n');
            assert_eq!(
                parse_argv(&input_line).unwrap(),
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
