use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeErrorKind {
    #[error("MissingStackValueError")]
    MissingValue,

    #[error("OperationNotSupportedError: {op} is not supported {target}")]
    OperationNotSupported { target: String, op: String },

    #[error("UndefinedVariableError: name '{name}' is not defined")]
    UndefinedVariable { name: String },

    #[error("AlreadyDefinedVariableError: name '{name}' is already defined")]
    AlreadyDefinedVariable { name: String },

    #[error("TypeError: expected {expected}, got {provided}")]
    TypeError { expected: String, provided: String },
}

#[derive(Error, Debug)]
#[error("Error while parsing")]
pub struct ParsingError {}

#[derive(Debug, Error)]
#[error("[line {line}] Runtime error:\n{kind}")]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub line: usize,
}
