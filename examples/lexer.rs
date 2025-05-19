use std::fs::read_to_string;

use clap::Parser;
use reqlang_expr::prelude::*;

fn main() {
    let args = Args::parse();

    let source = read_to_string(args.path.expect("should have a file path to open"))
        .expect("should be able to open file at path");

    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    eprintln!("{tokens:#?}");
}

#[derive(Parser, Debug)]
#[command(version, about = "Example CLI that lexes an expression in to tokens")]
struct Args {
    /// Path to expression file
    path: Option<String>,
}
