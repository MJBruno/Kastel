use crate::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LexErrorKind {
    UnexpectedCharacter,
    UnterminatedString,
    UnterminatedBlockComment,
    InvalidNumber,
    InvalidEscape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl LexError {
    pub fn new(
        kind: LexErrorKind,
        span: Span,
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
            line,
            column,
        }
    }
}
