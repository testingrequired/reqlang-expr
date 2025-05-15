use logos::Logos;
use std::ops::Range;

use crate::errors::{self, LexicalError};

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

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(usize, Token, usize), (LexicalError, Range<usize>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.pending.take() {
            return Some(Ok(token));
        }

        match self.inner.next()? {
            token => {
                let span = self.inner.span();
                Some(token.map(|t| (span.start, t, span.end)))
            }
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

    #[regex("[!?:]?[a-zA-Z][a-zA-Z0-9_]*", lex_identifier)]
    Identifier(String),
}

fn lex_identifier(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice.to_string()
}

impl Token {
    pub fn identifier(identifier: &str) -> Self {
        Token::Identifier(identifier.to_string())
    }
}
