use clap::Parser;
use reqlang_expr::{cliutil::read_in_source, prelude::*};

fn main() {
    let args = Args::parse();

    let source = read_in_source(args.path);

    let ast = parse(&source);

    eprintln!("{ast:#?}");
}

#[derive(Parser, Debug)]
#[command(version, about = "Example CLI that parses an expression in to its AST")]
struct Args {
    /// Path to expression file
    path: Option<String>,
}
