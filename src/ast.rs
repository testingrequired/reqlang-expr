use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(Box<ExprIdentifier>),
    Call(Box<ExprCall>),
    String(Box<ExprString>),
}

impl Expr {
    pub fn identifier(identifier: &str) -> Self {
        Self::Identifier(Box::new(ExprIdentifier::new(identifier)))
    }

    pub fn string(string: &str) -> Self {
        Self::String(ExprString::new(string).into())
    }

    pub fn call(callee: (Expr, Range<usize>), args: Vec<(Expr, Range<usize>)>) -> Self {
        Self::Call(Box::new(ExprCall { callee, args }))
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
    pub callee: (Expr, Range<usize>),
    pub args: Vec<(Expr, Range<usize>)>,
}
