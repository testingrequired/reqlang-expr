use clap::Parser;
use reqlang_expr::{
    cliutil::{parse_key_val, read_in_source},
    disassembler::Disassembler,
    prelude::*,
};

fn main() -> ExprResult<()> {
    let args = Args::parse();

    let source = read_in_source(args.path);

    let ast: Expr = parse(&source).expect("should parse successfully");

    let env = CompileTimeEnv::new(args.vars, args.prompts, args.secrets, vec![]);

    let bytecode = compile(&mut (ast, 0..source.len()), &env)?;

    let disassemble = Disassembler::new(&bytecode, &env);
    let disassembly = disassemble.disassemble();

    eprintln!("{disassembly}");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Example CLI that compiles then disassembles an expression"
)]
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

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, u8>)]
    builtins: Vec<(String, u8)>,
}
