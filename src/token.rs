#[allow(dead_code)]
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

    // Opérateurs arithmetique
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,

    //Assignment
    PlusEqual,

    //Opérateur de comparaison
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Bang,
    BangEqual,

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
    Question,

    //Controle
    Eof,
    Char,
}

#[derive(Debug, Clone, PartialEq,Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[allow(dead_code)]
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub line: usize,
    pub column: usize,
}
#[allow(dead_code)]
impl Token {
    pub fn new(kind: TokenKind, span: Span, line: usize, column: usize) -> Self {
        Self {
            kind,
            span,
            line,
            column,
        }
    }

    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start..self.span.end]
    }
}
