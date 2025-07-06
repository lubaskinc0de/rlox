use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeErrorKind {
    #[error("Missing stack value")]
    MissingValue,

    #[error("Operation {op} is not supported {value}")]
    OperationNotSupported { value: String, op: String },
}

#[derive(Debug, Error)]
#[error("Empty chunk error")]
pub struct EmptyChunkError {}

#[derive(Error, Debug)]
#[error("Error while parsing")]
pub struct ParsingError {}

#[derive(Debug, Error)]
#[error("[line {line}] Runtime error:\n{kind}")]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub line: usize,
}
