//! Abstract syntax tree types

use crate::{span::Spanned, types::Type};

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

    pub fn identifier_name(&self) -> Option<&str> {
        match self {
            Expr::Identifier(expr_identifier) => Some(expr_identifier.name()),
            _ => None,
        }
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

    pub fn is_bool(&self) -> bool {
        self.get_type() == Type::Bool
    }

    pub fn get_type(&self) -> Type {
        match self {
            Expr::Bool(_) => Type::Bool,
            Expr::Identifier(_) => Type::Unknown,
            Expr::Call(_) => Type::Unknown,
            Expr::String(_) => Type::String,
            Expr::Error => Type::Unknown,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ExprIdentifier(pub String);

impl ExprIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }

    pub fn name(&self) -> &str {
        &self.0
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
