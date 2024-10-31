use std::{error::Error, fmt};

#[derive(Debug)]
pub enum AnalyzerError {
    ParserError(String),
    DisallowedImport(String, String),
    ExceededLoopDepth(u32),
    NoBreakCondition,
    ExceededLoopIterations(u32),
    ExceededInstructionLimit(u32),
    ExceededCompUnitLimit(u64),
}

// Implement Display for custom error formatting
impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AnalyzerError::ParserError(ref message) => write!(f, "{}", message),
            AnalyzerError::DisallowedImport(ref module, ref imp) => {
                write!(f, "Disallowed import: {}::{}", module, imp)
            }
            AnalyzerError::ExceededLoopDepth(ref ld) => {
                write!(f, "Exceeded max loop depth of: {}", ld)
            }
            AnalyzerError::NoBreakCondition => write!(f, "No break condition found"),
            AnalyzerError::ExceededLoopIterations(ref it) => {
                write!(f, "Exceeded max loop iterations of: {}", it)
            }
            AnalyzerError::ExceededInstructionLimit(ref il) => {
                write!(f, "Exceeded max instruction limit of: {}", il)
            }
            AnalyzerError::ExceededCompUnitLimit(ref cl) => {
                write!(f, "Exceeded max computation unit limit of: {}", cl)
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
            AnalyzerError::ExceededLoopDepth(_) => None,
            AnalyzerError::NoBreakCondition => None,
            AnalyzerError::ExceededLoopIterations(_) => None,
            AnalyzerError::ExceededInstructionLimit(_) => None,
            AnalyzerError::ExceededCompUnitLimit(_) => None,
        }
    }
}
