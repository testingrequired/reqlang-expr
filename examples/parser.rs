use clap::Parser;
use reqlang_expr::{cli::read_in_source, prelude::*};

fn main() {
    let args = Args::parse();

    let source = read_in_source(args.path);

    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    let ast: Expr = ExprParser::new()
        .parse(tokens)
        .expect("should parse tokens to ast");

    eprintln!("{ast:#?}");
}

#[derive(Parser, Debug)]
#[command(version, about = "Example CLI that parses an expression in to its AST")]
struct Args {
    /// Path to expression file
    path: Option<String>,
}
