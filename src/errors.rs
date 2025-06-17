//! Errors

use std::ops::Range;

use thiserror::Error;

pub type ExprResult<T> = std::result::Result<T, Vec<(ExprError, Range<usize>)>>;

#[derive(Debug, Error, PartialEq)]
pub enum ExprError {
    #[error("There was an error lexing expression: {0}")]
    LexError(#[from] LexicalError),
    #[error("There was a compliation error with the expression: {0}")]
    CompileError(#[from] CompileError),
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexicalError {
    #[default]
    #[error("Invalid token")]
    InvalidToken,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompileError {
    #[error("undefined: {0}")]
    Undefined(String),
    #[error("expects {expected} arguments but received {actual}")]
    WrongNumberOfArgs { expected: usize, actual: usize },
}
