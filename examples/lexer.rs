use clap::Parser;
use reqlang_expr::{cliutil::read_in_source, prelude::*};

fn main() {
    let args = Args::parse();

    let source = read_in_source(args.path);

    let tokens = lex(&source);

    eprintln!("{tokens:#?}");
}

#[derive(Parser, Debug)]
#[command(version, about = "Example CLI that lexes an expression in to tokens")]
struct Args {
    /// Path to expression file
    path: Option<String>,
}
