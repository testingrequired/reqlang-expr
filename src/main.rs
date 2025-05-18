use std::{
    io::{Read, Write, stdin, stdout},
    process::exit,
};

use clap::Parser;
use reqlang_expr::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Read from stdin (instead of loading from a file)
    #[arg(long)]
    stdin: bool,

    /// Path to expression file (if not using stdin)
    path: Option<String>,

    /// Run the interpreter (instead of compiling)
    #[arg(long)]
    interpret: bool,

    /// Load bytecode instead of source code
    #[arg(long)]
    bytecode: bool,

    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    vars: Vec<String>,

    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    prompts: Vec<String>,

    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    secrets: Vec<String>,
}

fn main() {
    let args = Args::parse();

    if args.bytecode {
        let bytecode = if args.stdin {
            let mut bytecode = vec![];

            let _ = stdin().read_to_end(&mut bytecode).unwrap();

            bytecode
        } else {
            // Read from args.path
            let file = std::fs::read(args.path.clone().unwrap()).unwrap();

            file
        };

        let bytecode = ExprByteCode { codes: bytecode };

        eprintln!("Bytecode:\n\n{bytecode:#?}\n");

        if args.interpret {
            let mut vm: Vm<'_> = Vm::default();
            vm.interpret(&bytecode).expect("should interpret bytecode");

            exit(0);
        }

        let _ = stdout().write_all(&bytecode.codes);

        exit(0);
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

        let bytecode: ExprByteCode = compile(&ast, &env);

        eprintln!("Bytecode:\n\n{bytecode:#?}\n");

        if args.interpret {
            let mut vm: Vm<'_> = Vm::default();
            vm.interpret(&bytecode).expect("should interpret bytecode");

            exit(0);
        }

        let _ = stdout().write_all(&bytecode.codes);

        exit(0);
    }
}
