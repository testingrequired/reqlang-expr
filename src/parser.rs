//! Parsing source code in to expressions

use lalrpop_util::lalrpop_mod;

use crate::{
    ast::{self, add_type_to_expr_parse},
    errors::{ExprResult, SyntaxError},
    lexer::lex,
    parser::grammar::ExprParser,
};

lalrpop_mod!(grammar);

/// Parse source code in to an [`ast::Expr`].
pub fn parse(source: &str) -> ExprResult<ast::Expr> {
    let tokens = lex(source);

    let mut errs = vec![];

    let expr_parser = ExprParser::new();

    let mut parser_errors = Vec::new();

    let mut expr = match expr_parser.parse(source, &mut parser_errors, tokens) {
        Ok(ast) => ast,
        Err(err) => {
            errs.push(SyntaxError::from_parser_error(err, source));
            ast::Expr::Error
        }
    };

    add_type_to_expr_parse(&mut expr);

    errs.extend(parser_errors);

    if errs.is_empty() { Ok(expr) } else { Err(errs) }
}
