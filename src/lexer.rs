use crate::errors::ParseError;
use crate::token::Token;

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace();

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::End),
        };

        // Numbers
        if ch.is_ascii_digit() || ch == '.' {
            let start = self.pos;

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '.' {
                    self.advance();
                } else {
                    break;
                }
            }

            let value = self.input[start..self.pos]
                .parse::<f64>()
                .map_err(|_| ParseError::InvalidNumber)?;

            return Ok(Token::Number(value));
        }

        // Identifiers
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = self.pos;

            while let Some(c) = self.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.advance();
                } else {
                    break;
                }
            }

            return Ok(Token::Identifier(self.input[start..self.pos].to_string()));
        }

        // Operators
        let token = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '^' => Token::Caret,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            ',' => Token::Comma,
            other => return Err(ParseError::UnexpectedCharacter(other)),
        };

        self.advance();

        Ok(token)
    }
}
