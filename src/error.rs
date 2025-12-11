use thiserror::Error;

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Name resolution error: {0}")]
    NameResolution(String),
    
    #[error("Type error: {0}")]
    Type(String),
    
    #[error("IR emission error: {0}")]
    Emission(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
