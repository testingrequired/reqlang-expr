use std::ops::Range;

use lalrpop_util::lalrpop_mod;

use crate::{
    ast,
    errors::{ExprResult, SyntaxError},
    lexer::Lexer,
    parser::grammar::ExprParser,
    prelude::ExprError,
};

lalrpop_mod!(grammar);

/// Parse source code in to an [`ast::Expr`].
pub fn parse(source: &str) -> ExprResult<ast::Expr> {
    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    let mut errs = vec![];

    let expr_parser = ExprParser::new();

    let mut parser_errors = Vec::new();

    let expr = match expr_parser.parse(&mut parser_errors, tokens) {
        Ok(ast) => ast,
        Err(err) => {
            errs.push(err);
            ast::Expr::Error
        }
    };

    errs.extend(parser_errors);

    let errs: Vec<(ExprError, Range<usize>)> = errs
        .into_iter()
        .map(|err| SyntaxError::from_parser_error(err, &source))
        .collect();

    if errs.is_empty() { Ok(expr) } else { Err(errs) }
}
