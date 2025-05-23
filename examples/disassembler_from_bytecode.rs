use clap::Parser;
use reqlang_expr::{cli::parse_key_val, disassembler::Disassembler, prelude::*};

fn main() {
    let args = Args::parse();

    let env = Env::new(args.vars, args.prompts, args.secrets);

    let codes = std::fs::read(&args.path).expect("should be able to read source from file");
    let bytecode = ExprByteCode::from(codes);

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
