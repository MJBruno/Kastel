use crate::{
    error::{LexError, LexErrorKind},
    token::{Span, Token, TokenKind},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Lexer<'a> {
    source: &'a str,

    start: usize,
    current: usize,

    line: usize,
    column: usize,

    token_line: usize,
    token_column: usize,
}
#[allow(dead_code)]
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,

            start: 0,
            current: 0,

            line: 1,
            column: 1,

            token_line: 1,
            token_column: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        let ch = self.source[self.current..].chars().next()?;

        self.current += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column += 1;
        } else {
            self.column += 1;
        }

        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        self.source[self.current..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        let mut chars = self.source[self.current..].chars();

        chars.next()?;
        chars.next()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() != Some(expected) {
            return false;
        }

        self.advance();
        true
    }

    fn skip_whitespace(&mut self) -> Result<(), LexError> {
        loop {
            match self.peek() {
                Some(' ' | '\r' | '\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('/') if self.peek_next() == Some('*') => {
                    self.skip_block_comment()?;
                }

                _ => return Ok(()),
            }
        }
    }

    fn identifier(&mut self) -> TokenKind {
        while let Some(ch) = self.peek() {
            if !is_identifier_continue(ch) {
                break;
            }

            self.advance();
        }

        let text = &self.source[self.start..self.current];

        keyword_or_identifier(text)
    }
    fn number(&mut self) -> Result<TokenKind, LexError> {
        if self.source[self.start..self.current] == *"0" {
            match self.peek() {
                Some('x' | 'X') => {
                    self.advance();
                    if !self.consume_digits(16) {
                        return Err(self.invalid_number(self.peek().unwrap()));
                    }
                    return Ok(TokenKind::Number);
                }

                Some('b' | 'B') => {
                    self.advance();
                    if !self.consume_digits(2) {
                        return Err(self.invalid_number(self.peek().unwrap()));
                    }
                    return Ok(TokenKind::Number);
                }
                Some('o' | 'O') => {
                    self.advance();
                    if !self.consume_digits(8) {
                        return Err(self.invalid_number(self.peek().unwrap()));
                    }
                    return Ok(TokenKind::Number);
                }
                _ => {}
            }
        }

        if !self.consume_digits(10) {
            return Err(self.invalid_number(self.peek().unwrap()));
        }
        if self.peek() == Some('.') && self.peek_next().is_some_and(|ch| ch.is_ascii_digit()) {
            self.advance();

            if !self.consume_digits(10) {
                return Err(self.invalid_number(self.peek().unwrap()));
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }

            if !self.consume_digits(10) {
                return Err(self.invalid_number(self.peek().unwrap()));
            }
        }
        Ok(TokenKind::Number)
    }

    fn invalid_number(&self, e: char) -> LexError {
        LexError::new(
            LexErrorKind::UnterminatedBlockComment,
            Span {
                start: self.start,
                end: self.current,
            },
            format!("invalid numeric literal {e}"),
            self.line,
            self.column,
        )
    }

    fn string(&mut self) -> Result<TokenKind, LexError> {
        loop {
            let ch = match self.peek() {
                Some(ch) => ch,
                None => {
                    return Err(LexError::new(
                        LexErrorKind::UnterminatedString,
                        Span {
                            start: self.start,
                            end: self.current,
                        },
                        "unterminate string",
                        self.token_line,
                        self.token_column,
                    ));
                }
            };

            match ch {
                '"' => {
                    self.advance();
                    return Ok(TokenKind::String);
                }
                '\\' => {
                    self.advance();
                    self.consume_escape()?;
                }
                '\n' => {
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span::new(self.start, self.current),
            line: self.token_line,
            column: self.token_column,
        }
    }

    fn scan_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace()?;

        self.start = self.current;
        self.token_line = self.line;
        self.token_column = self.column;

        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        let ch = self.advance().unwrap();

        let kind = match ch {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBracket,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            ';' => TokenKind::Semicolon,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '\'' => self.character()?,

            '=' => {
                if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '"' => self.string()?,

            ch if ch.is_ascii_digit() => self.number()?,
            ch if is_identifier_start(ch) => self.identifier(),
            ch => {
                return Err(LexError::new(
                    LexErrorKind::UnexpectedCharacter,
                    Span {
                        start: self.start,
                        end: self.current,
                    },
                    format!("Unexpected character {ch}"),
                    self.token_line,
                    self.token_column,
                ));
            }
        };

        Ok(self.make_token(kind))
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<LexError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            match self.scan_token() {
                Ok(token) => {
                    let eof = token.kind == TokenKind::Eof;

                    tokens.push(token);
                    if eof {
                        break;
                    }
                }
                Err(error) => {
                    errors.push(error);
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let start = self.current;
        self.advance();
        self.advance();
        while !self.is_at_end() {
            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }

        Err(LexError::new(
            LexErrorKind::UnterminatedBlockComment,
            Span {
                start,
                end: self.current,
            },
            "unterminate block skip_line_comment ",
            self.line,
            self.column,
        ))
    }

    fn consume_digits(&mut self, base: u32) -> bool {
        let mut has_digit = false;
        let mut previous_was_separator = false;

        while let Some(ch) = self.peek() {
            if ch == '_' {
                if !has_digit || previous_was_separator {
                    break;
                }
                previous_was_separator = true;
                self.advance();
                continue;
            }

            if ch.is_digit(base) {
                has_digit = true;
                previous_was_separator = false;
                self.advance();
                continue;
            }

            break;
        }
        has_digit && !previous_was_separator
    }

    fn decimal_number(&mut self) -> TokenKind {
        self.consume_digits(10);
        TokenKind::Number
    }

    fn consume_escape(&mut self) -> Result<(), LexError> {
        let ch = match self.peek() {
            Some(ch) => ch,
            None => {
                return Err(LexError::new(
                    LexErrorKind::UnterminatedString,
                    Span {
                        start: self.start,
                        end: self.current,
                    },
                    "unterminate string after escape",
                    self.line,
                    self.column,
                ));
            }
        };

        if !is_valid_escape(ch) {
            return Err(LexError::new(
                LexErrorKind::InvalidEscape,
                Span {
                    start: self.start,
                    end: self.current,
                },
                format!("invalid escape sequence '\\{ch}'"),
                self.line,
                self.column,
            ));
        }

        self.advance();
        Ok(())
    }

    fn character(&mut self) -> Result<TokenKind, LexError> {
        if self.is_at_end() {
            return Err(LexError::new(
                LexErrorKind::UnterminatedString,
                Span {
                    start: self.start,
                    end: self.current,
                },
                "unterminate character literal",
                self.token_line,
                self.token_column,
            ));
        }

        match self.peek() {
            Some('\\') => {
                self.advance();
                self.consume_escape()?;
            }

            Some('\n') => {
                return Err(LexError::new(
                    LexErrorKind::UnterminatedString,
                    Span {
                        start: self.start,
                        end: self.current,
                    },
                    "newline in character literal",
                    self.line,
                    self.column,
                ));
            }
            Some('\'') => {
                return Err(LexError::new(
                    LexErrorKind::InvalidEscape,
                    Span {
                        start: self.start,
                        end: self.current,
                    },
                    "empty character literal",
                    self.line,
                    self.column,
                ));
            }

            Some(_) => {
                self.advance();
            }
            None => unreachable!(),
        }

        if self.peek() != Some('\'') {
            return Err(LexError::new(
                LexErrorKind::InvalidEscape,
                Span {
                    start: self.start,
                    end: self.current,
                },
                " character literal must contain exactly one character",
                self.line,
                self.column,
            ));
        }

        self.advance();
        Ok(TokenKind::Char)
    }
}
#[allow(dead_code)]
fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}
fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
fn keyword_or_identifier(text: &str) -> TokenKind {
    match text {
        "let" => TokenKind::Let,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "fun" => TokenKind::Function,
        "return" => TokenKind::Return,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "nil" => TokenKind::Nil,

        _ => TokenKind::Identifier,
    }
}
fn is_valid_escape(ch: char) -> bool {
    matches!(ch, 'n' | 'r' | 't' | '\\' | '"' | '\'' | '0')
}
