use std::collections::VecDeque;

use crate::parser::ParserError;

#[derive(Debug)]
pub enum Token {
    Word(String),
    StdoutRedirect,
}

pub struct Tokenizer {
    input: VecDeque<char>,
    tokens: Vec<Token>,
    buf: String,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            input: VecDeque::new(),
            tokens: Vec::new(),
            buf: String::new(),
        }
    }

    pub fn tokenize(&mut self, input: &str) -> Result<Vec<Token>, ParserError> {
        self.tokens.clear();
        self.input = input.chars().collect();
        self.collect_tokens()?;
        Ok(std::mem::take(&mut self.tokens))
    }

    fn collect_tokens(&mut self) -> Result<(), ParserError> {
        loop {
            self.skip_whitespace();
            self.buf.clear();

            let Some(c) = self.input.front().copied() else {
                break;
            };

            match c {
                '>' => {
                    self.input.pop_front();
                    self.tokens.push(Token::StdoutRedirect);
                }
                '1' => {
                    self.input.pop_front();
                    if self.input.front().is_some_and(|c| *c == '>') {
                        self.tokens.push(Token::StdoutRedirect);
                        self.input.pop_front();
                    } else {
                        self.buf.push(c);
                        self.tokenize_word()?;
                    }
                }
                _ => {
                    self.tokenize_word()?;
                }
            };
        }

        Ok(())
    }

    fn tokenize_word(&mut self) -> Result<(), ParserError> {
        loop {
            // if we reach the end of the input without a newline (e.g. by escaping the newline)
            // we need more input
            let Some(c) = self.input.pop_front() else {
                return Err(ParserError::NeedNextLine);
            };
            match c {
                ' ' | '\n' | '\t' => break,
                '>' | '1' => {
                    if self.handle_redirect_mid_word(c) {
                        return Ok(());
                    }
                }
                '\'' => self.tokenize_single_quoted()?,
                '"' => self.tokenize_double_quoted()?,
                '\\' => self.escape_next(),
                _ => self.buf.push(c),
            }
        }

        self.add_word_token();
        Ok(())
    }

    fn tokenize_single_quoted(&mut self) -> Result<(), ParserError> {
        let mut terminated = false;
        while let Some(c) = self.input.pop_front() {
            match c {
                '\'' => {
                    terminated = true;
                    break;
                }
                _ => self.buf.push(c),
            }
        }

        if !terminated {
            Err(ParserError::UnterminatedSingleQuoteString)
        } else {
            Ok(())
        }
    }

    fn tokenize_double_quoted(&mut self) -> Result<(), ParserError> {
        let mut terminated = false;

        while let Some(c) = self.input.pop_front() {
            match c {
                '"' => {
                    terminated = true;
                    break;
                }
                '\\' => {
                    let Some(cn) = self.input.front() else {
                        self.buf.push('\\');
                        break;
                    };
                    // only escape ", \, $, `, \n while in double quotes
                    match cn {
                        '"' | '\\' | '$' | '`' | '\n' => self.escape_next(),
                        _ => self.buf.push('\\'),
                    }
                }
                _ => self.buf.push(c),
            }
        }

        if !terminated {
            Err(ParserError::UnterminatedDoubleQuoteString)
        } else {
            Ok(())
        }
    }

    // helpers
    fn skip_whitespace(&mut self) {
        while self.input.front().is_some_and(|c| c.is_whitespace()) {
            self.input.pop_front();
        }
    }

    fn escape_next(&mut self) {
        if let Some(c) = self.input.pop_front() {
            self.buf.push(c);
        }
    }

    fn handle_redirect_mid_word(&mut self, c: char) -> bool {
        match c {
            '>' => {}
            '1' if self.input.front().is_some_and(|c| *c == '>') => {
                self.input.pop_front();
            }
            c => {
                self.buf.push(c);
                return false;
            }
        }
        self.add_word_token();
        self.tokens.push(Token::StdoutRedirect);
        true
    }

    fn add_word_token(&mut self) {
        if !self.buf.is_empty() {
            self.tokens.push(Token::Word(std::mem::take(&mut self.buf)));
        }
    }
}
