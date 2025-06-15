use std::rc::Rc;

use clap::Parser;
use reqlang_expr::{
    cliutil::{parse_key_val, read_in_source, unzip_key_values},
    prelude::*,
};

fn main() -> ExprResult<()> {
    let args = Args::parse();

    let source = read_in_source(args.path);

    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    let ast: Expr = ExprParser::new()
        .parse(tokens)
        .expect("should parse tokens to ast");

    let builtins = args
        .builtins
        .iter()
        .map(|builtin| {
            Rc::new(BuiltinFn {
                name: builtin.0.clone(),
                arity: FnArity::N(builtin.1),
                func: Rc::new(|_| "".into()),
            })
        })
        .collect::<Vec<_>>();

    let (var_keys, var_values) = unzip_key_values(args.vars);
    let (prompt_keys, prompt_values) = unzip_key_values(args.prompts);
    let (secret_keys, secret_values) = unzip_key_values(args.secrets);
    let (client_context_keys, client_context_values) = unzip_key_values(args.client_context);

    let mut env = CompileTimeEnv::new(var_keys, prompt_keys, secret_keys, client_context_keys);

    env.add_user_builtins(builtins);

    let bytecode = compile(&(ast, 0..source.len()), &env)?;

    let mut vm = Vm::new();

    let runtime_env: RuntimeEnv = RuntimeEnv {
        vars: var_values,
        prompts: prompt_values,
        secrets: secret_values,
        client_context: client_context_values
            .iter()
            .map(|string_value| Value::String(string_value.clone()))
            .collect(),
    };

    let value = vm.interpret(bytecode.into(), &env, &runtime_env)?;

    println!("{value}");

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about = "Example CLI that compiles an expression")]
struct Args {
    /// Path to expression file
    path: Option<String>,

    /// List of indexed variable names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, String>)]
    vars: Vec<(String, String)>,

    /// List of indexed prompt names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, String>)]
    prompts: Vec<(String, String)>,

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, String>)]
    secrets: Vec<(String, String)>,

    /// List of indexed builtin names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, u8>)]
    builtins: Vec<(String, u8)>,

    /// List of indexed client context names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, String>)]
    client_context: Vec<(String, String)>,
}
