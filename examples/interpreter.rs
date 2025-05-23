use std::rc::Rc;

use clap::Parser;
use reqlang_expr::{
    cli::{parse_key_val, read_in_source},
    prelude::*,
};

fn main() {
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
                arity: builtin.1,
                func: Rc::new(|_| "".into()),
            })
        })
        .collect::<Vec<_>>();

    let var_keys = args.vars.clone().into_iter().map(|(key, _)| key).collect();

    let var_values = args
        .vars
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let prompt_keys = args
        .prompts
        .clone()
        .into_iter()
        .map(|(key, _)| key)
        .collect();

    let prompt_values = args
        .prompts
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let secret_keys = args
        .secrets
        .clone()
        .into_iter()
        .map(|(key, _)| key)
        .collect();

    let secret_values = args
        .secrets
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let mut env = Env::new(var_keys, prompt_keys, secret_keys);

    env.add_builtins(builtins);

    let bytecode = compile(&ast, &env);

    let mut vm = Vm::new();

    let runtime_env: RuntimeEnv = RuntimeEnv {
        vars: var_values,
        prompts: prompt_values,
        secrets: secret_values,
    };

    let _ = vm.interpret(&bytecode, &env, &runtime_env);
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

    /// List of indexed secret names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, u8>)]
    builtins: Vec<(String, u8)>,
}
