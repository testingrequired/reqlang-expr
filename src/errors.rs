use thiserror::Error;

pub type ExprResult<T> = std::result::Result<T, Vec<ExprError>>;

#[derive(Debug, Error, PartialEq)]
pub enum ExprError {
    #[error("There was an error lexing expression: {0}")]
    LexError(#[from] LexicalError),
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexicalError {
    #[default]
    #[error("Invalid token")]
    InvalidToken,
}
