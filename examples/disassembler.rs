use std::fs::read_to_string;

use clap::Parser;
use reqlang_expr::{cli::parse_key_val, disassembler::Disassembler, prelude::*};

fn main() {
    let args = Args::parse();

    let source = read_to_string(args.path).expect("should be able to open file at path");

    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    let ast: Expr = ExprParser::new()
        .parse(tokens)
        .expect("should parse tokens to ast");

    let builtins = args.builtins.iter().map(|builtin| builtin.into()).collect();

    let env = Env {
        vars: args.vars.clone(),
        prompts: args.prompts.clone(),
        secrets: args.secrets.clone(),
        builtins,
        ..Default::default()
    };

    let bytecode = compile(&ast, &env);

    let disassemble = Disassembler::new(&bytecode, &env);
    let disassembly = disassemble.disassemble(None);

    eprintln!("{disassembly}");
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Example CLI that compiles then disassembles an expression"
)]
struct Args {
    /// Path to expression file
    path: String,

    /// List of indexed variable names
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    vars: Vec<String>,

    /// List of indexed prompt names
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    prompts: Vec<String>,

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    secrets: Vec<String>,

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, u8>)]
    builtins: Vec<(String, u8)>,
}
