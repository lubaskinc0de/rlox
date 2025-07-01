use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error compiling")]
pub struct CompileError {}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Missing value error")]
    MissingValue,

    #[error("Operation {operation_type} is not supported for value of type {value_type}")]
    OperationNotSupported {
        value_type: String,
        operation_type: String,
    },
}

#[derive(Debug, Error)]
#[error("Empty chunk error")]
pub struct EmptyChunkError {}
