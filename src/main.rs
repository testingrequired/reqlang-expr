use std::{
    fs::File,
    io::{Read, Write, stdin, stdout},
    process::exit,
};

use clap::Parser;
use reqlang_expr::prelude::*;

fn main() {
    let args = Args::parse();

    let bytecode: ExprByteCode = read_in_bytecode(&args);

    if args.interpret {
        interpret_bytecode(&bytecode);
    }

    write_out_bytecode(args, bytecode);
}

#[derive(Parser, Debug)]
#[command(version, about = "CLI utility for working with reqlang expressions")]
struct Args {
    /// Read from stdin (instead of loading from a file)
    #[arg(long)]
    stdin: bool,

    /// Path to expression file (if not using stdin)
    path: Option<String>,

    /// Out path to write bytecode to
    #[arg(long)]
    out_path: Option<String>,

    /// Interpret the bytecode and exit
    ///
    /// Instead of writing the bytecode out
    #[arg(long)]
    interpret: bool,

    /// Load bytecode instead of source code
    #[arg(long)]
    bytecode: bool,

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

fn read_in_bytecode(args: &Args) -> ExprByteCode {
    let bytecode = if args.bytecode {
        let bytecode = if args.stdin {
            let mut bytecode = vec![];

            stdin()
                .read_to_end(&mut bytecode)
                .expect("should be able to read bytecode from stdin");

            bytecode
        } else {
            std::fs::read(args.path.clone().unwrap())
                .expect("should be able to read source from file")
        };

        ExprByteCode { codes: bytecode }
    } else {
        let source = if args.stdin {
            let mut source = String::new();

            let _ = stdin().read_to_string(&mut source).unwrap();

            source
        } else {
            // Read from args.path
            let file = std::fs::read_to_string(args.path.clone().unwrap()).unwrap();

            file
        };

        eprintln!("Source:\n\n{source}\n");

        let lexer: Lexer<'_> = Lexer::new(&source);
        let tokens = lexer.collect::<Vec<_>>();

        eprintln!("Tokens:\n\n{tokens:#?}\n");

        let ast: Expr = ExprParser::new()
            .parse(tokens)
            .expect("should parse tokens to ast");

        eprintln!("AST:\n\n{ast:#?}\n");

        let env = Env {
            vars: args.vars.clone(),
            prompts: args.prompts.clone(),
            secrets: args.secrets.clone(),
            ..Default::default()
        };

        eprintln!("Env:\n\n{env:#?}\n");

        compile(&ast, &env)
    };

    eprintln!("Bytecode:\n\n{bytecode:#?}\n");

    bytecode
}

fn interpret_bytecode(bytecode: &ExprByteCode) {
    let mut vm: Vm<'_> = Vm::default();
    vm.interpret(bytecode).expect("should interpret bytecode");

    exit(0);
}

fn write_out_bytecode(args: Args, bytecode: ExprByteCode) {
    if let Some(out_path) = args.out_path {
        let mut file = File::create(out_path).expect("should create output file");

        file.write_all(&bytecode.codes)
            .expect("should write bytecode to output file");
    } else {
        let _ = stdout().write_all(&bytecode.codes);

        exit(0);
    }
}
