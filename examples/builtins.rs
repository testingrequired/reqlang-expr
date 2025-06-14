use std::rc::Rc;

use reqlang_expr::prelude::*;

fn main() -> ExprResult<()> {
    let source = "(id2 (id (id2 (noop))))";

    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    let ast: Expr = ExprParser::new()
        .parse(tokens)
        .expect("should parse tokens to ast");

    let builtins = vec![Rc::new(BuiltinFn {
        name: "id2".to_string(),
        arity: 1,
        func: Rc::new(|args: Vec<Value>| {
            let arg = args.get(0).unwrap();

            arg.get_string().into()
        }),
    })];

    let var_keys = vec!["a".to_string()];

    let var_values = vec!["a_value".to_string()];

    let mut env = Env::new(var_keys, vec![], vec![]);

    env.add_user_builtins(builtins);

    let bytecode = compile(&ast, &env)?;

    let mut vm = Vm::new();

    let runtime_env: RuntimeEnv = RuntimeEnv {
        vars: var_values,
        ..Default::default()
    };

    let result = vm
        .interpret(bytecode.into(), &env, &runtime_env)
        .expect("should be ok");

    assert_eq!("noop", result.get_string());

    Ok(())
}
