// ================================================================
// PARSE_ERROR
// ================================================================

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}
