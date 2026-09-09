use crate::error::lex_error::LexerError;
use crate::frontend::token::Token;
use crate::frontend::token::TokenKind;

pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    errors: Vec<LexerError>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token::new(kind, lexeme.to_string(), self.line, self.column - 1)
    }

    fn is_identifier_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_identifier(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn identifier(&mut self, first: char) -> Token {
        let start_column = self.column - 1;

        let mut text = String::new();
        text.push(first);

        while Self::is_identifier(self.peek()) {
            text.push(self.advance());
        }

        let kind = match Token::keyword(&text) {
            Some(keyword) => keyword,
            None => TokenKind::Identifier,
        };

        Token::new(kind, text, self.line, start_column)
    }

    pub fn scan_token(&mut self) -> Result<Vec<Token>, Vec<LexerError>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let c = self.advance();

            match c {
                // ====================================================
                // ESPACES
                // ====================================================
                ' ' | '\t' | '\r' | '\n' => {}

                // ====================================================
                // IDENTIFIANTS
                // ====================================================
                c if Self::is_identifier_start(c) => {
                    tokens.push(self.identifier(c));
                }

                // ====================================================
                // NOMBRES
                // ====================================================
                c if c.is_ascii_digit() => {
                    tokens.push(self.number(c));
                }

                // ====================================================
                // CHAÎNES
                // ====================================================
                '"' => {
                    tokens.push(self.string());
                }

                // ====================================================
                // PARENTHÈSES
                // ====================================================
                '(' => {
                    tokens.push(self.make_token(TokenKind::LeftParen, "("));
                }

                ')' => {
                    tokens.push(self.make_token(TokenKind::RightParen, ")"));
                }

                // ====================================================
                // ACCOLADES
                // ====================================================
                '{' => {
                    tokens.push(self.make_token(TokenKind::LeftBrace, "{"));
                }

                '}' => {
                    tokens.push(self.make_token(TokenKind::RightBrace, "}"));
                }

                // ====================================================
                // CROCHETS
                // ====================================================
                '[' => {
                    tokens.push(self.make_token(TokenKind::LeftBracket, "["));
                }

                ']' => {
                    tokens.push(self.make_token(TokenKind::RightBracket, "]"));
                }

                // ====================================================
                // PONCTUATION
                // ====================================================
                ',' => {
                    tokens.push(self.make_token(TokenKind::Comma, ","));
                }

                '.' => {
                    tokens.push(self.make_token(TokenKind::Dot, "."));
                }

                ':' => {
                    tokens.push(self.make_token(TokenKind::Colon, ":"));
                }

                ';' => {
                    tokens.push(self.make_token(TokenKind::Semicolon, ";"));
                }

                '?' => {
                    tokens.push(self.make_token(TokenKind::Question, "?"));
                }

                // ====================================================
                // +
                // +=
                // ====================================================
                '+' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::PlusEqual, "+="));
                    } else {
                        tokens.push(self.make_token(TokenKind::Plus, "+"));
                    }
                }

                // ====================================================
                // -
                // -=
                // ====================================================
                '-' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::MinusEqual, "-="));
                    } else {
                        tokens.push(self.make_token(TokenKind::Minus, "-"));
                    }
                }

                // ====================================================
                // *
                // *=
                // ====================================================
                '*' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::StarEqual, "*="));
                    } else {
                        tokens.push(self.make_token(TokenKind::Star, "*"));
                    }
                }

                // ====================================================
                // /
                // /=
                // //
                // /*
                // ====================================================
                '/' => {
                    if self.match_char('/') {
                        self.skip_comment();
                    } else if self.match_char('*') {
                        self.skip_multiline_comment();
                    } else if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::SlashEqual, "/="));
                    } else {
                        tokens.push(self.make_token(TokenKind::Slash, "/"));
                    }
                }

                // ====================================================
                // %
                // %=
                // ====================================================
                '%' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::PercentEqual, "%="));
                    } else {
                        tokens.push(self.make_token(TokenKind::Percent, "%"));
                    }
                }
                #[allow(unreachable_patterns)]
                // ====================================================
                // =
                // ==
                // =>
                // ====================================================
                '.' => {
                    if self.match_char('.') {
                        if self.match_char('=') {
                            tokens.push(self.make_token(TokenKind::RangeInclusive, "..="));
                        } else {
                            tokens.push(self.make_token(TokenKind::Range, ".."));
                        }
                    } else {
                        tokens.push(self.make_token(TokenKind::Dot, "."));
                    }
                }
                // ====================================================
                // =
                // ==
                // =>
                // ====================================================
                '=' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::EqualEqual, "=="));
                    } else if self.match_char('>') {
                        tokens.push(self.make_token(TokenKind::FatArrow, "=>"));
                    } else {
                        tokens.push(self.make_token(TokenKind::Equal, "="));
                    }
                }

                // ====================================================
                // !
                // !=
                // ====================================================
                '!' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::NotEqual, "!="));
                    } else {
                        tokens.push(self.make_token(TokenKind::Not, "!"));
                    }
                }

                // ====================================================
                // <
                // <=
                // <<
                // ====================================================
                '<' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::LessEqual, "<="));
                    } else if self.match_char('<') {
                        tokens.push(self.make_token(TokenKind::LeftShift, "<<"));
                    } else {
                        tokens.push(self.make_token(TokenKind::Less, "<"));
                    }
                }

                // ====================================================
                // >
                // >=
                // >>
                // ====================================================
                '>' => {
                    if self.match_char('=') {
                        tokens.push(self.make_token(TokenKind::GreaterEqual, ">="));
                    } else if self.match_char('>') {
                        tokens.push(self.make_token(TokenKind::RightShift, ">>"));
                    } else {
                        tokens.push(self.make_token(TokenKind::Greater, ">"));
                    }
                }

                // ====================================================
                // &
                // &&
                // ====================================================
                '&' => {
                    if self.match_char('&') {
                        tokens.push(self.make_token(TokenKind::And, "&&"));
                    } else {
                        tokens.push(self.make_token(TokenKind::Ampersand, "&"));
                    }
                }

                // ====================================================
                // |
                // ||
                // ====================================================
                '|' => {
                    if self.match_char('|') {
                        tokens.push(self.make_token(TokenKind::Or, "||"));
                    } else {
                        tokens.push(self.make_token(TokenKind::Pipe, "|"));
                    }
                }

                // ====================================================
                // BITWISE
                // ====================================================
                '^' => {
                    tokens.push(self.make_token(TokenKind::Caret, "^"));
                }

                '~' => {
                    tokens.push(self.make_token(TokenKind::Tilde, "~"));
                }

                // ====================================================
                // ERREUR
                // ====================================================
                _ => {
                    self.errors.push(LexerError {
                        message: format!("Caractère inattendu '{}'", c),
                        line: self.line,
                        column: self.column - 1,
                    });
                }
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            String::new(),
            self.line,
            self.column,
        ));

        if self.errors.is_empty() {
            Ok(tokens)
        } else {
            Err(self.errors.clone())
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];

        self.current += 1;

        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn number(&mut self, first: char) -> Token {
        let start_column = self.column - 1;

        // ========================================================
        // HEXADÉCIMAL
        // ========================================================
        if first == '0' && (self.peek() == 'x' || self.peek() == 'X') {
            self.advance();

            let mut digits = String::new();

            while self.peek().is_ascii_hexdigit() || self.peek() == '_' {
                let c = self.advance();

                if c != '_' {
                    digits.push(c);
                }
            }

            let value = u64::from_str_radix(&digits, 16).unwrap_or(0);

            return Token::new(
                TokenKind::Number,
                value.to_string(),
                self.line,
                start_column,
            );
        }

        // ========================================================
        // BINAIRE
        // ========================================================
        if first == '0' && (self.peek() == 'b' || self.peek() == 'B') {
            self.advance();

            let mut digits = String::new();

            while self.peek() == '0' || self.peek() == '1' || self.peek() == '_' {
                let c = self.advance();

                if c != '_' {
                    digits.push(c);
                }
            }

            let value = u64::from_str_radix(&digits, 2).unwrap_or(0);

            return Token::new(
                TokenKind::Number,
                value.to_string(),
                self.line,
                start_column,
            );
        }

        // ========================================================
        // DÉCIMAL
        // ========================================================
        let mut text = String::new();

        text.push(first);

        while self.peek().is_ascii_digit() || self.peek() == '_' {
            let c = self.advance();

            if c != '_' {
                text.push(c);
            }
        }

        // ========================================================
        // FLOTTANT
        // ========================================================
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            text.push(self.advance());

            while self.peek().is_ascii_digit() || self.peek() == '_' {
                let c = self.advance();

                if c != '_' {
                    text.push(c);
                }
            }
        }

        // ========================================================
        // EXPOSANT
        // ========================================================
        if self.peek() == 'e' || self.peek() == 'E' {
            let exponent_starts_number = self.peek_next().is_ascii_digit()
                || self.peek_next() == '+'
                || self.peek_next() == '-';

            if exponent_starts_number {
                text.push(self.advance());

                if self.peek() == '+' || self.peek() == '-' {
                    text.push(self.advance());
                }

                while self.peek().is_ascii_digit() {
                    text.push(self.advance());
                }
            }
        }

        Token::new(TokenKind::Number, text, self.line, start_column)
    }

    fn string(&mut self) -> Token {
        let start_column = self.column - 1;

        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let c = self.advance();

            if c == '\\' {
                let escaped = self.advance();

                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    _ => value.push(escaped),
                }
            } else {
                value.push(c);
            }
        }

        if !self.is_at_end() {
            self.advance();
        } else {
            self.errors.push(LexerError {
                message: "Chaîne non fermée".to_string(),
                line: self.line,
                column: self.column,
            });
        }

        Token::new(TokenKind::String, value, self.line, start_column)
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn skip_multiline_comment(&mut self) {
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                return;
            }

            self.advance();
        }

        self.errors.push(LexerError {
            message: "Commentaire bloc non fermé".to_string(),
            line: self.line,
            column: self.column,
        });
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.advance();

        true
    }
}
