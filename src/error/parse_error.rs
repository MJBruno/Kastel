

// ================================================================
// PARSE_ERROR
// ================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}