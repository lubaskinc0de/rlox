use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeErrorKind {
    #[error("OperationNotSupportedError: {op} is not supported {target}")]
    OperationNotSupported { target: String, op: String },

    #[error("UndefinedVariableError: name '{name}' is not defined")]
    UndefinedVariable { name: String },

    #[error("AlreadyDefinedVariableError: name '{name}' is already defined")]
    AlreadyDefinedVariable { name: String },

    #[error("TypeError: expected {expected}, got {provided}")]
    TypeError { expected: String, provided: String },

    #[error("CallError: {func}() expected {arity} arguments but got {arg_count}")]
    CallError {
        func: String,
        arg_count: usize,
        arity: usize,
    },

    #[error("Stack overflow")]
    StackOverflow,
}

#[derive(Error, Debug)]
#[error("Error while parsing")]
pub struct ParsingError {}

#[derive(Debug, Error)]
#[error("[line {line}] Runtime error:\n{kind}\nTraceback (most recent call last):\n{traceback}")]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub line: usize,
    pub traceback: String,
}
