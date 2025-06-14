use std::rc::Rc;

use clap::Parser;
use once_cell::sync::Lazy;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use regex::Regex;
use reqlang_expr::{
    cli::{parse_key_val, unzip_key_values},
    disassembler::Disassembler,
    prelude::*,
};

fn main() -> ExprResult<()> {
    let args = Args::parse();

    let mut line_editor = Reedline::create();

    let mut prompt = DefaultPrompt::default();
    prompt.left_prompt = DefaultPromptSegment::Basic("interpet    ".to_string());

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

    let (mut var_keys, mut var_values) = unzip_key_values(args.vars);
    let (mut prompt_keys, mut prompt_values) = unzip_key_values(args.prompts);
    let (mut secret_keys, mut secret_values) = unzip_key_values(args.secrets);

    let mut repl_mode = ReplMode::default();

    let mut last_value: Option<Value> = None;

    loop {
        let mut env = Env::new(var_keys.clone(), prompt_keys.clone(), secret_keys.clone());

        env.add_builtins(builtins.clone());

        let runtime_env: RuntimeEnv = RuntimeEnv {
            vars: var_values.clone(),
            prompts: prompt_values.clone(),
            secrets: secret_values.clone(),
        };

        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(mut source)) => {
                if source.trim().is_empty() {
                    continue;
                }

                if EXIT_PATTERN.is_match(&source) {
                    break;
                }

                if MODE_GET_PATTERN.is_match(&source) {
                    println!("MODE: {repl_mode:#?}");
                    continue;
                }

                if MODE_SET_PATTERN.is_match(&source) {
                    for (_, [new_mode]) in
                        MODE_SET_PATTERN.captures_iter(&source).map(|c| c.extract())
                    {
                        match new_mode {
                            "interpret" => {
                                repl_mode = ReplMode::Interpret;
                                prompt.left_prompt =
                                    DefaultPromptSegment::Basic("interpet    ".to_string());
                            }
                            "compile" => {
                                repl_mode = ReplMode::Compile;
                                prompt.left_prompt =
                                    DefaultPromptSegment::Basic("compile     ".to_string());
                            }
                            "disassemble" => {
                                repl_mode = ReplMode::Disassemble;
                                prompt.left_prompt =
                                    DefaultPromptSegment::Basic("disassemble ".to_string());
                            }
                            "parse" => {
                                repl_mode = ReplMode::Parse;
                                prompt.left_prompt =
                                    DefaultPromptSegment::Basic("parse       ".to_string());
                            }
                            "lex" => {
                                repl_mode = ReplMode::Lex;
                                prompt.left_prompt =
                                    DefaultPromptSegment::Basic("lex         ".to_string());
                            }
                            _ => {
                                println!(
                                    "Invalid repl mode: '{new_mode}'. Please use 'interpret', 'compile', 'disassemble', 'parse', or 'lex'\n"
                                );
                            }
                        }
                    }

                    continue;
                }

                if ENV_PATTERN.is_match(&source) {
                    println!("{env:#?}");
                    continue;
                }

                if let Some(last_value) = &last_value {
                    source = source.replace(REPL_LAST_VALUE_PLACEHOLDER, &last_value.to_string());
                } else if source.contains(REPL_LAST_VALUE_PLACEHOLDER) {
                    println!("No expression has been interpeted yet.");
                    continue;
                }

                if SET_PATTERN.is_match(&source) {
                    for (_, [set_type, key, value]) in
                        SET_PATTERN.captures_iter(&source).map(|c| c.extract())
                    {
                        match set_type {
                            "var" => {
                                var_keys.push(key.to_string());
                                var_values.push(value.to_string());
                            }
                            "prompt" => {
                                prompt_keys.push(key.to_string());
                                prompt_values.push(value.to_string());
                            }
                            "secret" => {
                                secret_keys.push(key.to_string());
                                secret_values.push(value.to_string());
                            }
                            _ => {}
                        }
                    }

                    continue;
                }

                let lexer: Lexer<'_> = Lexer::new(&source);
                let tokens = lexer.collect::<Vec<_>>();

                if repl_mode == ReplMode::Lex {
                    println!("{tokens:#?}");
                    continue;
                }

                let ast = ExprParser::new().parse(tokens);

                match ast {
                    Ok(ast) => {
                        if repl_mode == ReplMode::Parse {
                            println!("{ast:#?}");
                            continue;
                        }

                        let bytecode = compile(&ast, &env)?;

                        if repl_mode == ReplMode::Compile {
                            println!("{bytecode:#?}");
                            continue;
                        }

                        if repl_mode == ReplMode::Disassemble {
                            let disassemble = Disassembler::new(&bytecode, &env);
                            let disassembly = disassemble.disassemble(None);

                            println!("{disassembly}");
                            continue;
                        }

                        match vm.interpret(bytecode.into(), &env, &runtime_env) {
                            Ok(value) => {
                                println!("{value}");

                                if matches!(value, Value::String(_)) {
                                    last_value = Some(value);
                                }
                            }
                            Err(err) => {
                                println!("{err:#?}");
                            }
                        }
                    }
                    Err(err) => {
                        println!("{err:#?}");
                    }
                }
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

    Ok(())
}

static REPL_LAST_VALUE_PLACEHOLDER: &'static str = "%";

/// # Set Command
///
/// ```repl
/// /set {var|prompt|secret} key = value
/// ```
///
/// Set a variable, prompt, or secret with a given value
static SET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"/set (var|prompt|secret) ([a-zA-Z]+) = (.*)").expect(INVALID_REGEX_ERROR)
});

/// # Env Command
///
/// ```repl
/// /env
/// ```
///
/// Print out the current environment
///
/// - builtin functions
/// - variables
/// - prompts
/// - secrets
static ENV_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"/env").expect(INVALID_REGEX_ERROR));

/// # Exit Command
///
/// ```repl
/// /exit
/// ```
///
/// Exit the REPL
static EXIT_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^/exit$").expect(INVALID_REGEX_ERROR));

/// # Set Mode Command
///
/// ```repl
/// /mode compile
/// ```
///
/// Set the REPL's current [`ReplMode`]:
///
/// - `interpret` (default)
/// - `dissassemble`
/// - `compile`
/// - `parse`
/// - `lex`
static MODE_SET_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^/mode (.+)$").expect(INVALID_REGEX_ERROR));

/// # Get Mode Command
///
/// ```repl
/// /mode
/// ```
///
/// Get the REPL's current [`ReplMode`]:
///
/// - `interpret` (default)
/// - `dissassemble`
/// - `compile`
/// - `parse`
/// - `lex`
static MODE_GET_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^/mode$").expect(INVALID_REGEX_ERROR));

static INVALID_REGEX_ERROR: &str = "should be a valid regex pattern";

/// Controls what the repl does with input
#[derive(PartialEq, Debug, Default)]
enum ReplMode {
    #[default]
    Interpret,
    Compile,
    Disassemble,
    Parse,
    Lex,
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
