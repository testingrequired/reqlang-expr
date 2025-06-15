//! The lexer and associated types

use logos::Logos;
use std::ops::Range;

use crate::errors::{self, ExprError};

/// Converts a [`String`] source in to a vector of [`Token`]
#[derive(Debug)]
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
    pending: Option<(usize, Token, usize)>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
            pending: None,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(usize, Token, usize), (ExprError, Range<usize>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.pending.take() {
            return Some(Ok(token));
        }

        let token = self.inner.next()?;

        {
            let Range { start, end } = self.inner.span();

            Some(
                token
                    .map(|token| (start, token, end))
                    .map_err(|(err, err_span)| (err.into(), err_span)),
            )
        }
    }
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = (errors::LexicalError, Range<usize>))]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("(")]
    LParan,

    #[token(")")]
    RParan,

    #[regex(r#"`[^`]*`"#, lex_string)]
    String(String),

    #[regex("[!?:@]?[a-zA-Z_][a-zA-Z0-9_]*", lex_identifier)]
    Identifier(String),

    #[token("true")]
    True,

    #[token("false")]
    False,
}

fn lex_identifier(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice.to_string()
}

fn lex_string(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice[1..slice.len() - 1].to_string()
}

impl Token {
    pub fn identifier(identifier: &str) -> Self {
        Token::Identifier(identifier.to_string())
    }
}
