use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Missing value error")]
    MissingValue,
}

#[derive(Debug, Error)]
#[error("Empty chunk error")]
pub struct EmptyChunkError {}
