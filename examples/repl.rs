use clap::Parser;
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self};
use nu_ansi_term::Color;
use once_cell::sync::Lazy;
use reedline::{
    ColumnarMenu, DefaultCompleter, DefaultPrompt, DefaultPromptSegment, Emacs, ExampleHighlighter,
    KeyCode, KeyModifiers, MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal,
    default_emacs_keybindings,
};
use regex::Regex;
use reqlang_expr::{
    cliutil::{parse_key_val, unzip_key_values},
    disassembler::Disassembler,
    errors::diagnostics::get_diagnostics,
    prelude::*,
};

fn main() -> ExprResult<()> {
    let args = Args::parse();

    let mut prompt = DefaultPrompt::default();
    prompt.left_prompt = DefaultPromptSegment::Basic("interpet    ".to_string());

    // Set up the required keybindings
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    let mut commands = vec![
        "/env".into(),
        "/exit".into(),
        "/mode".into(),
        "/mode interpret".into(),
        "/mode compile".into(),
        "/mode disassemble".into(),
        "/mode lex".into(),
        "/mode parse".into(),
        "/set var ".into(),
        "/set prompt ".into(),
        "/set secret ".into(),
        "id".into(),
        "noop".into(),
        "is_empty".into(),
        "not".into(),
        "and".into(),
        "or".into(),
        "cond".into(),
        "to_str".into(),
        "concat".into(),
        "contains".into(),
        "trim".into(),
        "trim_start".into(),
        "trim_end".into(),
        "lowercase".into(),
        "uppercase".into(),
        "eq".into(),
        "type".into(),
    ];

    // Diagnostics
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();

    let mut vm = Vm::new();

    let (mut var_keys, mut var_values) = unzip_key_values(args.vars);
    let (mut prompt_keys, mut prompt_values) = unzip_key_values(args.prompts);
    let (mut secret_keys, mut secret_values) = unzip_key_values(args.secrets);
    let (mut client_keys, mut client_values) = unzip_key_values(args.client_context);

    {
        commands.extend(
            var_keys
                .iter()
                .map(|key| format!(":{}", key))
                .collect::<Vec<String>>(),
        );
        commands.extend(
            prompt_keys
                .iter()
                .map(|key| format!("?{}", key))
                .collect::<Vec<String>>(),
        );
        commands.extend(
            secret_keys
                .iter()
                .map(|key| format!("!{}", key))
                .collect::<Vec<String>>(),
        );
        commands.extend(
            client_keys
                .iter()
                .map(|key| format!("@{}", key))
                .collect::<Vec<String>>(),
        );
    }

    let mut repl_mode = ReplMode::default();
    let mut last_value: Option<Value> = None;

    loop {
        let mut env = CompileTimeEnv::new(
            var_keys.clone(),
            prompt_keys.clone(),
            secret_keys.clone(),
            client_keys.clone(),
        );

        let mut runtime_env: RuntimeEnv = RuntimeEnv {
            vars: var_values.clone(),
            prompts: prompt_values.clone(),
            secrets: secret_values.clone(),
            client_context: client_values
                .iter()
                .map(|string_value| Value::String(string_value.clone()))
                .collect(),
        };

        if let Some(last_value) = &last_value {
            let i = env.add_to_client_context(REPL_LAST_VALUE_PLACEHOLDER);
            runtime_env.add_to_client_context(i, last_value.clone());
            commands.extend(vec![format!("@{}", REPL_LAST_VALUE_PLACEHOLDER)]);
        }

        // Use the interactive menu to select options from the completer
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

        let edit_mode = Box::new(Emacs::new(keybindings.clone()));

        let mut completions = DefaultCompleter::with_inclusions(&['/', ':', '?', '!', '@', '_']);
        completions.insert(commands.clone());

        let mut example_highlighter = ExampleHighlighter::new(commands.clone());
        example_highlighter.change_colors(Color::Yellow, Color::White, Color::LightGray);

        let mut line_editor = Reedline::create()
            .with_completer(Box::new(completions.clone()))
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_edit_mode(edit_mode)
            .with_highlighter(Box::new(example_highlighter));

        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(source)) => {
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

                if SET_PATTERN.is_match(&source) {
                    for (_, [set_type, key, value]) in
                        SET_PATTERN.captures_iter(&source).map(|c| c.extract())
                    {
                        match set_type {
                            "var" => {
                                var_keys.push(key.to_string());
                                var_values.push(value.to_string());
                                commands.extend(vec![format!(":{}", key.to_string())]);
                            }
                            "prompt" => {
                                prompt_keys.push(key.to_string());
                                prompt_values.push(value.to_string());
                                commands.extend(vec![format!("?{}", key.to_string())]);
                            }
                            "secret" => {
                                secret_keys.push(key.to_string());
                                secret_values.push(value.to_string());
                                commands.extend(vec![format!("!{}", key.to_string())]);
                            }
                            "client" => {
                                client_keys.push(key.to_string());
                                client_values.push(value.to_string());
                                commands.extend(vec![format!("@{}", key.to_string())]);
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

                        let bytecode = compile(&(ast, 0..source.len()), &env);

                        match bytecode {
                            Ok(bytecode) => {
                                if repl_mode == ReplMode::Compile {
                                    println!("{bytecode:#?}");
                                    continue;
                                }

                                if repl_mode == ReplMode::Disassemble {
                                    let disassemble = Disassembler::new(&bytecode, &env);
                                    let disassembly = disassemble.disassemble();

                                    println!("{disassembly}");
                                    continue;
                                }

                                match vm.interpret(bytecode.into(), &env, &runtime_env) {
                                    Ok(value) => {
                                        println!("{value}");

                                        last_value = Some(value);
                                    }
                                    Err(err) => {
                                        let diagnostics = get_diagnostics(&err, &source);

                                        let file = SimpleFile::new("expression", source);

                                        for diagnostic in diagnostics {
                                            term::emit(
                                                &mut writer.lock(),
                                                &config,
                                                &file,
                                                &diagnostic,
                                            )
                                            .expect("should emit diagnostics to term");
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                let diagnostics = get_diagnostics(&err, &source);

                                let file = SimpleFile::new("expression", source);

                                for diagnostic in diagnostics {
                                    term::emit(&mut writer.lock(), &config, &file, &diagnostic)
                                        .expect("should emit diagnostics to term");
                                }
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

static REPL_LAST_VALUE_PLACEHOLDER: &str = "_";

/// # Set Command
///
/// ```repl
/// /set {var|prompt|secret} key = value
/// ```
///
/// Set a variable, prompt, or secret with a given value
static SET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"/set (var|prompt|secret|client) ([a-zA-Z]+) = (.*)").expect(INVALID_REGEX_ERROR)
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

    /// List of indexed client context names
    #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser=parse_key_val::<String, String>)]
    client_context: Vec<(String, String)>,
}
