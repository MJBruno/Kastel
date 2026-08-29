// ================================================================
// LEXER_ERROR
// ================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}