use clap::Parser;
use reqlang_expr::cliutil::unzip_key_values;
use reqlang_expr::{cliutil::parse_key_val, prelude::*};
use rstest::rstest;
use std::{fs::read_to_string, path::PathBuf};

#[rstest]
fn spec_files_tokens(#[files("spec/**/*.expr")] path: PathBuf) -> ExprResult<()> {
    let expected_tokens_path = path.with_extension("expr.tokens");

    if expected_tokens_path.exists() {
        let expected_tokens =
            read_to_string(expected_tokens_path).expect("should be able to read file");

        let expr_source = read_to_string(path).expect("should be able to read file");

        let tokens = lex(&expr_source);

        pretty_assertions::assert_eq!(format!("{tokens:#?}"), expected_tokens);
    }

    Ok(())
}

#[rstest]
fn spec_files_disassembled(#[files("spec/**/*.expr")] path: PathBuf) -> ExprResult<()> {
    let expected_disassembled_path = path.with_extension("expr.disassembled");
    let expr_source = read_to_string(path).expect("should be able to read file");

    if expected_disassembled_path.exists() {
        use reqlang_expr::disassembler::Disassembler;

        let expected_disassembled =
            read_to_string(expected_disassembled_path).expect("should be able to read file");

        let (args, expected_disassembled) = if let Some((i, _)) = expected_disassembled
            .lines()
            .next()
            .filter(|line| line.starts_with("//"))
            .and_then(|line| Some((line.len() + 1, line)))
        {
            expected_disassembled.split_at(i)
        } else {
            ("", expected_disassembled.as_str())
        };

        let args = if args.is_empty() {
            vec![]
        } else {
            let args: Vec<&str> = args[2..].trim().split_whitespace().collect();
            let mut args_with_empty_prefix = Vec::with_capacity(args.len() + 1);
            args_with_empty_prefix.push("");
            args_with_empty_prefix.extend(args);

            args_with_empty_prefix
        };

        let args = Args::try_parse_from(args).expect("should parse args");

        let (var_keys, _) = unzip_key_values(args.vars);
        let (prompt_keys, _) = unzip_key_values(args.prompts);
        let (secret_keys, _) = unzip_key_values(args.secrets);
        let (client_context_keys, _) = unzip_key_values(args.client_context);

        let env = CompileTimeEnv::new(var_keys, prompt_keys, secret_keys, client_context_keys);

        match parse(&expr_source) {
            Ok(ast) => match compile(&mut (ast, 0..expr_source.len()), &env) {
                Ok(bytecode) => {
                    let disassemble = Disassembler::new(&bytecode, &env);
                    let disassembly = disassemble.disassemble();

                    pretty_assertions::assert_eq!(expected_disassembled, disassembly);
                }
                Err(err) => {
                    pretty_assertions::assert_eq!(expected_disassembled, format!("{err:#?}"));
                }
            },
            Err(err) => {
                pretty_assertions::assert_eq!(expected_disassembled, format!("{err:#?}"));
            }
        }
    }

    Ok(())
}

#[rstest]
fn spec_files_interpreted(#[files("spec/**/*.expr")] path: PathBuf) -> ExprResult<()> {
    let expected_iterpreted_path = path.with_extension("expr.interpreted");
    let expr_source = read_to_string(path).expect("should be able to read file");

    if expected_iterpreted_path.exists() {
        let expected_interpreted =
            read_to_string(expected_iterpreted_path).expect("should be able to read file");

        let (args, expected_interpreted) = if let Some((i, _)) = expected_interpreted
            .lines()
            .next()
            .filter(|line| line.starts_with("//"))
            .and_then(|line| Some((line.len() + 1, line)))
        {
            expected_interpreted.split_at(i)
        } else {
            ("", expected_interpreted.as_str())
        };

        let args = if args.is_empty() {
            vec![]
        } else {
            let args: Vec<&str> = args[2..].trim().split_whitespace().collect();
            let mut args_with_empty_prefix = Vec::with_capacity(args.len() + 1);
            args_with_empty_prefix.push("");
            args_with_empty_prefix.extend(args);

            args_with_empty_prefix
        };

        let args = Args::try_parse_from(args).expect("should parse args");

        let (var_keys, var_values) = unzip_key_values(args.vars);
        let (prompt_keys, prompt_values) = unzip_key_values(args.prompts);
        let (secret_keys, secret_values) = unzip_key_values(args.secrets);
        let (client_context_keys, client_context_values) = unzip_key_values(args.client_context);

        let env = CompileTimeEnv::new(var_keys, prompt_keys, secret_keys, client_context_keys);

        match parse(&expr_source) {
            Ok(ast) => match compile(&mut (ast, 0..expr_source.len()), &env) {
                Ok(bytecode) => {
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

                    match vm.interpret(bytecode.into(), &env, &runtime_env) {
                        Ok(value) => {
                            pretty_assertions::assert_eq!(value.to_string(), expected_interpreted);
                        }
                        Err(err) => {
                            pretty_assertions::assert_eq!(
                                format!("{err:#?}"),
                                expected_interpreted
                            );
                        }
                    }
                }
                Err(err) => {
                    pretty_assertions::assert_eq!(format!("{err:#?}"), expected_interpreted);
                }
            },
            Err(err) => {
                pretty_assertions::assert_eq!(format!("{err:#?}"), expected_interpreted);
            }
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command()]
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

    /// List of indexed client context names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, String>)]
    client_context: Vec<(String, String)>,
}
