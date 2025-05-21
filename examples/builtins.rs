use std::rc::Rc;

use reqlang_expr::prelude::*;

fn main() {
    let source = "(id (id (id :a)))";

    let lexer: Lexer<'_> = Lexer::new(&source);
    let tokens = lexer.collect::<Vec<_>>();

    let ast: Expr = ExprParser::new()
        .parse(tokens)
        .expect("should parse tokens to ast");

    let builtins = vec![Rc::new(BuiltinFn {
        name: "id".to_string(),
        arity: 1,
        func: Rc::new(|args: Vec<Value>| {
            let arg = args.get(0).unwrap();

            match arg {
                Value::String(value) => value.to_string(),
                Value::Fn(_) => panic!("id should not be called with a function"),
            }
        }),
    })];

    let var_keys = vec!["a".to_string()];

    let var_values = vec!["a_value".to_string()];

    let env = Env {
        vars: var_keys,
        builtins,
        ..Default::default()
    };

    let bytecode = compile(&ast, &env);

    let mut vm = Vm::new();

    let runtime_env: RuntimeEnv = RuntimeEnv {
        vars: var_values,
        ..Default::default()
    };

    let result = vm
        .interpret(&bytecode, &env, &runtime_env)
        .expect("should be ok");

    assert_eq!("a_value", result.get_string());
}
