use std::{
    fs::File,
    io::{Read, Write, stdin, stdout},
    process::exit,
    rc::Rc,
};

use clap::Parser;
use reqlang_expr::{cliutil::parse_key_val, prelude::*};

fn main() -> ExprResult<()> {
    let args = Args::parse();

    let builtins = args
        .builtins
        .iter()
        .map(|builtin| {
            Rc::new(BuiltinFn {
                name: builtin.0.clone(),
                arity: builtin.1,
                func: Rc::new(|_| "".into()),
            })
        })
        .collect();

    let mut env = CompileTimeEnv::new(
        args.vars.clone(),
        args.prompts.clone(),
        args.secrets.clone(),
    );

    env.add_user_builtins(builtins);

    eprintln!("Env:\n\n{env:#?}\n");

    let bytecode: Box<ExprByteCode> = read_in_bytecode(&args, &env)?.into();

    if bytecode.codes().is_empty() {
        println!("No bytecode found");
        exit(1);
    }

    if args.interpret {
        interpret_bytecode(bytecode.clone(), &env);
    }

    write_out_bytecode(args, bytecode);

    Ok(())
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

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, u8>)]
    builtins: Vec<(String, u8)>,
}

fn read_in_bytecode(args: &Args, env: &CompileTimeEnv) -> ExprResult<ExprByteCode> {
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

        Ok(ExprByteCode::new(bytecode, vec![]))
    } else {
        let source = if args.stdin {
            let mut source = String::new();

            stdin().read_to_string(&mut source).unwrap();

            source
        } else {
            std::fs::read_to_string(
                args.path
                    .clone()
                    .expect("should have a file path to open or pass --stdin flag"),
            )
            .expect("should be able to open file at path")
        };

        eprintln!("Source:\n\n{source}\n");

        let lexer: Lexer<'_> = Lexer::new(&source);
        let tokens = lexer.collect::<Vec<_>>();

        eprintln!("Tokens:\n\n{tokens:#?}\n");

        let ast: Expr = ExprParser::new()
            .parse(tokens)
            .expect("should parse tokens to ast");

        eprintln!("AST:\n\n{ast:#?}\n");

        compile(&ast, env)
    };

    eprintln!("Bytecode:\n\n{bytecode:#?}\n");

    bytecode
}

fn interpret_bytecode(bytecode: Box<ExprByteCode>, env: &CompileTimeEnv) {
    let mut vm = Vm::new();

    let value = vm
        .interpret(bytecode, env, &RuntimeEnv::default())
        .expect("should interpret bytecode");

    println!("{value}");

    exit(0);
}

fn write_out_bytecode(args: Args, bytecode: Box<ExprByteCode>) {
    if let Some(out_path) = args.out_path {
        let mut file = File::create(out_path).expect("should create output file");

        file.write_all(bytecode.codes())
            .expect("should write bytecode to output file");
    } else {
        let _ = stdout().write_all(bytecode.codes());

        exit(0);
    }
}
