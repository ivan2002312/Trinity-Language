use crate::ast::Span;
use std::fmt;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String, pub line: usize, pub column: usize, pub span: Span,
}

#[derive(Debug, Clone)]
pub enum CompilerError {
    LexerError { message: String, location: SourceLocation },
    ParserError { message: String, location: SourceLocation, expected: Vec<String>, found: String },
    RuntimeError { message: String },
    IOError(String),
}

impl CompilerError {
    pub fn lexer_error(m: &str, l: SourceLocation) -> Self { CompilerError::LexerError { message: m.to_string(), location: l } }
    pub fn parser_error(m: &str, l: SourceLocation, e: Vec<String>, f: &str) -> Self { CompilerError::ParserError { message: m.to_string(), location: l, expected: e, found: f.to_string() } }
    pub fn runtime_error(m: &str) -> Self { CompilerError::RuntimeError { message: m.to_string() } }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::LexerError { message, location } => write!(f, "Lexer error at {}:{}: {}", location.line, location.column, message),
            CompilerError::ParserError { message, location, expected, found } => write!(f, "Parser error at {}:{}: {} (expected {}, found {})", location.line, location.column, message, expected.join(", "), found),
            CompilerError::RuntimeError { message } => write!(f, "Runtime error: {}", message),
            CompilerError::IOError(e) => write!(f, "I/O: {}", e),
        }
    }
}
