use crate::parser::{ParserError, ShellInput};

#[derive(Debug)]
pub enum Token {
    Word(String),
}

pub fn get_next_token<'a>(input: &mut ShellInput<'a>) -> Result<Option<Token>, ParserError> {
    skip_whitespace(input);

    if input.peek().is_none() {
        Ok(None)
    } else {
        Ok(Some(tokenize_word(input)?))
    }
}

fn skip_whitespace<'a>(input: &mut ShellInput<'a>) {
    while input.peek().is_some_and(|c| c.is_whitespace()) {
        input.next();
    }
}

fn tokenize_word<'a>(input: &mut ShellInput<'a>) -> Result<Token, ParserError> {
    let mut buf = String::new();
    loop {
        // if we reach the end of the input without a newline (e.g. by escaping the newline)
        // we need more input
        let Some(c) = input.next() else {
            return Err(ParserError::NeedNextLine);
        };
        match c {
            ' ' | '\n' | '\t' => break,
            '\'' => tokenize_single_quoted(input, &mut buf)?,
            '"' => tokenize_double_quoted(input, &mut buf)?,
            '\\' => escape_next(input, &mut buf),
            _ => buf.push(c),
        }
    }

    Ok(Token::Word(buf))
}

fn tokenize_single_quoted<'a>(
    input: &mut ShellInput<'a>,
    buf: &mut String,
) -> Result<(), ParserError> {
    let mut terminated = false;
    for c in input {
        match c {
            '\'' => {
                terminated = true;
                break;
            }
            _ => buf.push(c),
        }
    }

    if !terminated {
        Err(ParserError::UnterminatedSingleQuoteString)
    } else {
        Ok(())
    }
}

fn tokenize_double_quoted<'a>(
    input: &mut ShellInput<'a>,
    buf: &mut String,
) -> Result<(), ParserError> {
    let mut terminated = false;

    while let Some(c) = input.next() {
        match c {
            '"' => {
                terminated = true;
                break;
            }
            '\\' => {
                let Some(cn) = input.peek() else {
                    buf.push('\\');
                    break;
                };
                // only escape ", \, $, `, \n while in double quotes
                match cn {
                    '"' | '\\' | '$' | '`' | '\n' => escape_next(input, buf),
                    _ => buf.push('\\'),
                }
            }
            _ => buf.push(c),
        }
    }

    if !terminated {
        Err(ParserError::UnterminatedDoubleQuoteString)
    } else {
        Ok(())
    }
}

fn escape_next<'a>(input: &mut ShellInput<'a>, buf: &mut String) {
    if let Some(c) = input.next() {
        buf.push(c);
    }
}
