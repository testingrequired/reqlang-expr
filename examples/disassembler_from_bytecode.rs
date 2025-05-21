use std::rc::Rc;

use clap::Parser;
use reqlang_expr::{cli::parse_key_val, disassembler::Disassembler, prelude::*};

fn main() {
    let args = Args::parse();

    let builtins = args
        .builtins
        .iter()
        .map(|builtin| {
            Rc::new(BuiltinFn {
                name: builtin.0.clone(),
                arity: builtin.1,
                func: Rc::new(|_| String::new()),
            })
        })
        .collect();

    let env = Env {
        vars: args.vars.clone(),
        prompts: args.prompts.clone(),
        secrets: args.secrets.clone(),
        builtins,
        ..Default::default()
    };

    let codes = std::fs::read(&args.path).expect("should be able to read source from file");
    let bytecode = ExprByteCode { codes };

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
