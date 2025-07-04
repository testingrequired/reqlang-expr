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
            Expr::Identifier(expr_identifier) => Some(expr_identifier.lookup_name()),
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
pub struct ExprIdentifier(pub String, pub IdentifierKind, pub Option<Type>);

impl ExprIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(
            identifier.to_string(),
            Self::get_identifier_kind(identifier),
            None,
        )
    }

    pub fn get_identifier_kind(identifier: &str) -> IdentifierKind {
        let identifier_prefix = &identifier[..1];

        match identifier_prefix {
            "?" => IdentifierKind::Prompt,
            "!" => IdentifierKind::Secret,
            ":" => IdentifierKind::Var,
            "@" => IdentifierKind::Client,
            _ => IdentifierKind::Builtin,
        }
    }

    /// The full name of the identifier from the source code
    ///
    /// This is different from [Self::name] as it always includes a sigil prefix
    /// for variables, prompts, secrets, and client identifiers.
    pub fn full_name(&self) -> &str {
        &self.0
    }

    /// The look up name for the identifier
    ///
    /// For builtins this is just the identifier name
    ///
    /// For variables, prompts, secrets, and client identifiers, it returns the
    /// non sigil prefix version of the identifier.
    ///
    /// - builtin_fn => builtin_fn
    /// - :variable => variable
    /// - ?prompt => prompt
    /// - !secret => secret
    /// - @client => client
    ///
    pub fn lookup_name(&self) -> &str {
        match self.identifier_kind() {
            IdentifierKind::Builtin => &self.0,
            IdentifierKind::Var => &self.0[1..],
            IdentifierKind::Prompt => &self.0[1..],
            IdentifierKind::Secret => &self.0[1..],
            IdentifierKind::Client => &self.0[1..],
        }
    }

    pub fn sigil(&self) -> &str {
        &self.0[..1]
    }

    pub fn identifier_kind(&self) -> &IdentifierKind {
        &self.1
    }

    pub fn get_type(&self) -> &Option<Type> {
        &self.2
    }
}

#[derive(Debug, PartialEq)]
pub enum IdentifierKind {
    Builtin,
    Var,
    Prompt,
    Secret,
    Client,
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
