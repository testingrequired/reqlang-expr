//! Abstract syntax tree types

use crate::span::Spanned;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Bool(Box<ExprBool>),
    Identifier(Box<ExprIdentifier>),
    Call(Box<ExprCall>),
    String(Box<ExprString>),
    Error,
}

impl Expr {
    pub fn identifier(identifier: &str) -> Self {
        Self::Identifier(Box::new(ExprIdentifier::new(identifier)))
    }

    pub fn string(string: &str) -> Self {
        Self::String(ExprString::new(string).into())
    }

    pub fn call(callee: ExprS, args: Vec<ExprS>) -> Self {
        Self::Call(Box::new(ExprCall { callee, args }))
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(Box::new(ExprBool::new(value)))
    }
}

#[derive(Debug, PartialEq)]
pub struct ExprIdentifier(pub String);

impl ExprIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct ExprString(pub String);

impl ExprString {
    pub fn new(string: &str) -> Self {
        Self(string.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct ExprCall {
    pub callee: ExprS,
    pub args: Vec<ExprS>,
}

#[derive(Debug, PartialEq)]
pub struct ExprBool(pub bool);

impl ExprBool {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
}

pub type ExprS = Spanned<Expr>;
