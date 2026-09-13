#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ============================================================
    // LITTÉRAUX
    // ============================================================
    Identifier,
    Number,
    String,

    // ============================================================
    // MOTS-CLÉS
    // ============================================================
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Match,
    True,
    False,
    Nil,
    Break,
    Continue,
    Import,
    From,
    As,
    Export,
    // ============================================================
    // CLASSES
    // ============================================================
    Class,
    New,
    This,
    Base,
    Interface,
    // ============================================================
    // EXCEPTIONS
    // ============================================================
    Try,
    Catch,
    Throw,
    Finally,

    // ============================================================
    // OPÉRATEURS
    // ============================================================
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Question,

    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,

    Equal,
    EqualEqual,
    NotEqual,

    Less,
    LessEqual,

    Greater,
    GreaterEqual,

    // Arrow function:
    //
    // x => x * 2
    // (x, y) => x + y
    FatArrow,

    And,
    Or,
    Not,
    Is,
    // ============================================================
    // PATTERN MATCHING
    // ============================================================
    Range,
    RangeInclusive,

    // ============================================================
    // BITWISE
    // ============================================================
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    LeftShift,
    RightShift,

    // ============================================================
    // PONCTUATION
    // ============================================================
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
            "const" => Some(TokenKind::Const),
            "func" => Some(TokenKind::Function),
            "return" => Some(TokenKind::Return),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "match" => Some(TokenKind::Match),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "null" => Some(TokenKind::Nil),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "import" => Some(TokenKind::Import),
            "from" => Some(TokenKind::From),
            "as" => Some(TokenKind::As),
            "export" => Some(TokenKind::Export),
            // ========================================================
            // CLASSES
            // ========================================================
            "class" => Some(TokenKind::Class),
            "new" => Some(TokenKind::New),
            "this" => Some(TokenKind::This),
            "base" => Some(TokenKind::Base),
            "interface" => Some(TokenKind::Interface),
            // ========================================================
            // EXCEPTIONS
            // ========================================================
            "try" => Some(TokenKind::Try),
            "catch" => Some(TokenKind::Catch),
            "throw" => Some(TokenKind::Throw),
            "finally" => Some(TokenKind::Finally),

            "is" => Some(TokenKind::Is),

            _ => None,
        }
    }
}
