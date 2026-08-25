use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Littéraux
    Identifier,
    Number,
    String,

    // Mots-clés
    Let,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    True,
    False,
    Nil,
    Break,
    Continue,
    Print,

    // Opérateurs
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Question,

    Equal,

    EqualEqual,
    NotEqual,

    Less,
    LessEqual,

    Greater,
    GreaterEqual,

    And,
    Or,
    Not,

    // Ponctuation
    LeftParen,
    RightParen,

    LeftBrace,
    RightBrace,

    LeftBracket,
    RightBracket,
    Dot,
    Comma,
    Colon,
    Semicolon,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }

    pub fn keyword(text: &str) -> Option<TokenKind> {
        match text {
            "let" => Some(TokenKind::Let),
            "function" => Some(TokenKind::Function),
            "return" => Some(TokenKind::Return),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "for" => Some(TokenKind::For),
            "print" => Some(TokenKind::Print),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "nil" => Some(TokenKind::Nil),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),

            _ => None,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} '{}' ({}:{})",
            self.kind, self.lexeme, self.line, self.column
        )?;
        Ok(())
    }
}
