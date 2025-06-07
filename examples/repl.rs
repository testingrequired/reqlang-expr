use std::rc::Rc;

use clap::Parser;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use regex::Regex;
use reqlang_expr::{cli::parse_key_val, disassembler::Disassembler, prelude::*};

fn main() {
    let mut repl_mode = ReplMode::default();

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

    let vars: &Vec<(String, String)> = &args.vars;
    let mut var_keys: Vec<String> = vars.clone().into_iter().map(|(key, _)| key).collect();
    let mut var_values: Vec<String> = vars.clone().into_iter().map(|(_, value)| value).collect();

    let prompts: &Vec<(String, String)> = &args.prompts;
    let mut prompt_keys: Vec<String> = prompts.clone().into_iter().map(|(key, _)| key).collect();
    let mut prompt_values: Vec<String> = prompts
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let secrets: &Vec<(String, String)> = &args.secrets;
    let mut secret_keys: Vec<String> = secrets.clone().into_iter().map(|(key, _)| key).collect();
    let mut secret_values: Vec<String> = secrets
        .clone()
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    let set_pattern = Regex::new(SET_COMMAND_PATTERN).unwrap();
    let env_pattern = Regex::new(ENV_COMMAND_PATTERN).unwrap();
    let mode_set_pattern = Regex::new(MODE_SET_COMMAND_PATTERN).unwrap();
    let mode_get_pattern = Regex::new(MODE_GET_COMMAND_PATTERN).unwrap();

    loop {
        let sig = line_editor.read_line(&prompt);

        let mut env = Env::new(var_keys.clone(), prompt_keys.clone(), secret_keys.clone());

        env.add_builtins(builtins.clone());

        let runtime_env: RuntimeEnv = RuntimeEnv {
            vars: var_values.clone(),
            prompts: prompt_values.clone(),
            secrets: secret_values.clone(),
        };

        match sig {
            Ok(Signal::Success(source)) => {
                if source.trim().is_empty() {
                    continue;
                }

                if mode_get_pattern.is_match(&source) {
                    println!("MODE: {repl_mode:#?}");
                    continue;
                }

                if mode_set_pattern.is_match(&source) {
                    for (_, [new_mode]) in
                        mode_set_pattern.captures_iter(&source).map(|c| c.extract())
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

                if env_pattern.is_match(&source) {
                    println!("{env:#?}");
                    continue;
                }

                if set_pattern.is_match(&source) {
                    for (_, [set_type, key, value]) in
                        set_pattern.captures_iter(&source).map(|c| c.extract())
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

                let ast: Expr = ExprParser::new()
                    .parse(tokens)
                    .expect("should parse tokens to ast");

                if repl_mode == ReplMode::Parse {
                    println!("{ast:#?}");
                    continue;
                }

                let bytecode = compile(&ast, &env);

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

static SET_COMMAND_PATTERN: &str = r"/set (var|prompt|secret) ([a-zA-Z]+) = (.*)";
static ENV_COMMAND_PATTERN: &str = r"/env";
static MODE_SET_COMMAND_PATTERN: &str = r"^/mode (.+)$";
static MODE_GET_COMMAND_PATTERN: &str = r"^/mode$";

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
