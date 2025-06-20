use std::io::{Write, stdout};

use clap::Parser;
use reqlang_expr::{cliutil::read_in_source, prelude::*};

fn main() -> ExprResult<()> {
    let args = Args::parse();

    let source = read_in_source(args.path);

    let ast: Expr = parse(&source).expect("should parse successfully");

    let env = CompileTimeEnv::new(args.vars.clone(), vec![], vec![], vec![]);

    let bytecode = compile(&(ast, 0..source.len()), &env)?;

    eprintln!("{bytecode:#?}");

    let _ = stdout().write_all(bytecode.codes());

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about = "Example CLI that compiles an expression")]
struct Args {
    /// Path to expression file
    path: Option<String>,

    /// List of indexed variable names
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    vars: Vec<String>,

    /// List of indexed prompt names
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    prompts: Vec<String>,

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    secrets: Vec<String>,
}
