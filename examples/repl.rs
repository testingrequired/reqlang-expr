use std::rc::Rc;

use clap::Parser;
use reedline::{DefaultPrompt, Reedline, Signal};
use reqlang_expr::{cli::parse_key_val, prelude::*};

fn main() {
    let args = Args::parse();

    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

    let mut vm = Vm::new();

    let builtins: &Vec<(String, u8)> = &args.builtins;
    let builtins = builtins
        .iter()
        .map(|builtin| {
            Rc::new(BuiltinFn {
                name: builtin.0.clone(),
                arity: builtin.1,
                func: Rc::new(|_| "".into()),
            })
        })
        .collect::<Vec<_>>();

    let vars: &Vec<(String, String)> = &args.vars;
    let var_keys: Vec<String> = vars.clone().into_iter().map(|(key, _)| key).collect();
    let var_values: Vec<String> = vars.clone().into_iter().map(|(_, value)| value).collect();

    let prompts: &Vec<(String, String)> = &args.prompts;
    let prompt_keys: Vec<String> = prompts.clone().into_iter().map(|(key, _)| key).collect();
    let prompt_values: Vec<String> = prompts
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let secrets: &Vec<(String, String)> = &args.secrets;
    let secret_keys: Vec<String> = secrets.clone().into_iter().map(|(key, _)| key).collect();
    let secret_values: Vec<String> = secrets
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let mut env = Env::new(var_keys, prompt_keys, secret_keys);

    env.add_builtins(builtins);

    let runtime_env: RuntimeEnv = RuntimeEnv {
        vars: var_values.clone(),
        prompts: prompt_values.clone(),
        secrets: secret_values.clone(),
    };

    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::Success(source)) => {
                let lexer: Lexer<'_> = Lexer::new(&source);
                let tokens = lexer.collect::<Vec<_>>();

                let ast: Expr = ExprParser::new()
                    .parse(tokens)
                    .expect("should parse tokens to ast");

                let bytecode = compile(&ast, &env);

                let _ = vm.interpret(bytecode.into(), &env, &runtime_env);
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nAborted!");
                break;
            }
            x => {
                println!("Event: {:?}", x);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "REPL to compile expressions")]
struct Args {
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
