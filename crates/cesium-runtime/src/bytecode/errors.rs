use std::{error::Error, fmt};

#[derive(Debug)]
pub enum AnalyzerError {
    ParserError(String),
    DisallowedImport(String, String),
}

// Implement Display for custom error formatting
impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AnalyzerError::ParserError(ref message) => write!(f, "{}", message),
            AnalyzerError::DisallowedImport(ref module, ref imp) => {
                write!(f, "Disallowed import: {}::{}", module, imp)
            }
        }
    }
}

// Implement the Error trait for custom error handling
impl Error for AnalyzerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            AnalyzerError::ParserError(_) => None,
            AnalyzerError::DisallowedImport(_, _) => None,
        }
    }
}
