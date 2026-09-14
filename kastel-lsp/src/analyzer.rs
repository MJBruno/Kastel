use kastel::error::lex_error::LexerError;
use kastel::error::parse_error::ParserError;
use kastel::frontend::ast::Statement;
use kastel::frontend::lexer::Lexer;
use kastel::frontend::parser::Parser;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct AnalysisResult {
    pub statements: Vec<Statement>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn analyze(source: &str) -> AnalysisResult {
    let mut lexer = Lexer::new(source.to_owned());

    let tokens = match lexer.scan_token() {
        Ok(tokens) => tokens,
        Err(errors) => {
            return AnalysisResult {
                statements: Vec::new(),
                diagnostics: errors.into_iter().map(lexer_diagnostic).collect(),
            };
        }
    };

    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(statements) => AnalysisResult {
            statements,
            diagnostics: Vec::new(),
        },

        Err(errors) => AnalysisResult {
            statements: Vec::new(),
            diagnostics: errors.into_iter().map(parser_diagnostic).collect(),
        },
    }
}

fn lexer_diagnostic(error: LexerError) -> Diagnostic {
    Diagnostic {
        message: error.message,
        line: error.line,
        column: error.column,
    }
}

fn parser_diagnostic(error: ParserError) -> Diagnostic {
    Diagnostic {
        message: error.message,
        line: error.line,
        column: error.column,
    }
}
