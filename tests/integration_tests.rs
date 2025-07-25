use reqlang_expr::prelude::get_version_bytes;

macro_rules! test {
    (
        $source:tt;
        scenario: $test_name:ident $( $test_name2:ident)*;
        tokens should be: $expected_tokens:expr;
        ast should be: $expected_ast:expr;
        env: $env:tt;
        user builtins: $builtins:tt;
        compiles to: $expected_op_codes:expr;
        disassembles to: $expected_disassembly:expr;
        runtime env: $runtime_env:tt;
        interpets to: $expected_interpretation:expr;
    ) => {
        ::pastey::paste! {
            mod [< $test_name:lower $(_ $test_name2:lower)* _tests >] {
                use reqlang_expr::prelude::*;

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _tokens >]() {
                    let tokens = ::reqlang_expr::lexer::lex($source);

                    ::pretty_assertions::assert_eq!($expected_tokens, tokens);
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _ast >]() {
                    let ast = ::reqlang_expr::parser::parse(&$source);

                    ::pretty_assertions::assert_eq!($expected_ast, ast);
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _op_codes >]() {
                    let mut env: CompileTimeEnv = CompileTimeEnv::new$env;

                    env.add_user_builtins(vec!$builtins);
                    env.add_to_client_context("intest");

                    match ::reqlang_expr::parser::parse(&$source) {
                        Ok(ast) => {
                            let op_codes = ::reqlang_expr::compiler::compile(&mut (ast, 0..$source.len()), &env);
                            let expected_op_codes: ::reqlang_expr::errors::ExprResult<ExprByteCode> = $expected_op_codes;
                            ::pretty_assertions::assert_eq!(expected_op_codes, op_codes);
                        }
                        Err(err) => {
                            ::pretty_assertions::assert_eq!($expected_op_codes, ExprResult::<ExprByteCode>::Err(err));
                        }
                    };
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _op_codes_disassemble_to >]() {
                    let mut env: CompileTimeEnv = ::reqlang_expr::compiler::CompileTimeEnv::new$env;

                    env.add_user_builtins(vec!$builtins);
                    env.add_to_client_context("intest");

                    let ast = ::reqlang_expr::parser::parse(&$source);

                    if let Ok(ast) = ast && let Ok(op_codes) = ::reqlang_expr::compiler::compile(&mut (ast, 0..$source.len()), &env) {
                        let expected_disassembly: String = $expected_disassembly.to_string();
                        let disassemble = ::reqlang_expr::disassembler::Disassembler::new(&op_codes, &env);
                        let disassembly = disassemble.disassemble();

                        ::pretty_assertions::assert_eq!(expected_disassembly, disassembly);
                    }
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _interprets_without_error >]() {
                    let mut env: CompileTimeEnv = CompileTimeEnv::new$env;

                    env.add_user_builtins(vec!$builtins);
                    let i = env.add_to_client_context("intest");

                    match ::reqlang_expr::parser::parse(&$source) {
                        Ok(ast) => {
                            let op_codes = ::reqlang_expr::compiler::compile(&mut (ast, 0..$source.len()), &env);

                            match op_codes {
                                Ok(op_codes) => {
                                    let mut vm = Vm::new();
                                    let mut runtime_env: RuntimeEnv = RuntimeEnv$runtime_env;

                                    runtime_env.add_to_client_context(i, Value::Bool(true));

                                    let value = vm.interpret(op_codes.into(), &env, &runtime_env);

                                    let expected_interpretation: ::reqlang_expr::errors::ExprResult<Value> = $expected_interpretation;
                                    ::pretty_assertions::assert_eq!(expected_interpretation, value);
                                },
                                Err(err) => {
                                    let expected_interpretation: ::reqlang_expr::errors::ExprResult<Value> = $expected_interpretation;
                                    ::pretty_assertions::assert_eq!(expected_interpretation, Err(err));
                                }
                            }
                        }
                        Err(err) => {
                            let expected_interpretation: ::reqlang_expr::errors::ExprResult<Value> = $expected_interpretation;
                            ::pretty_assertions::assert_eq!(expected_interpretation, Err(err));
                        }
                    };
                }
            }
        }
    };

    (
        $source:tt;
        scenario: $test_name:ident $( $test_name2:ident)*;
        env: $env:tt;
        user builtins: $builtins:tt;
        runtime env: $runtime_env:tt;
        interpets to: $expected_interpretation:expr;
    ) => {
        ::pastey::paste! {
            mod [< $test_name:lower $(_ $test_name2:lower)* _tests >] {
                use reqlang_expr::prelude::*;

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _interprets_without_error >]() {
                    let mut env: CompileTimeEnv = CompileTimeEnv::new$env;

                    env.add_user_builtins(vec!$builtins);
                    let i = env.add_to_client_context("intest");

                    match ::reqlang_expr::parser::parse(&$source) {
                        Ok(ast) => {
                            let op_codes = ::reqlang_expr::compiler::compile(&mut (ast, 0..$source.len()), &env);

                            match op_codes {
                                Ok(op_codes) => {
                                    let mut vm = Vm::new();
                                    let mut runtime_env: RuntimeEnv = RuntimeEnv$runtime_env;

                                    runtime_env.add_to_client_context(i, Value::Bool(true));

                                    let value = vm.interpret(op_codes.into(), &env, &runtime_env);

                                    let expected_interpretation: ::reqlang_expr::errors::ExprResult<Value> = $expected_interpretation;
                                    ::pretty_assertions::assert_eq!(expected_interpretation, value);
                                },
                                Err(err) => {
                                    let expected_interpretation: ::reqlang_expr::errors::ExprResult<Value> = $expected_interpretation;
                                    ::pretty_assertions::assert_eq!(expected_interpretation, Err(err));
                                }
                            }
                        }
                        Err(err) => {
                            let expected_interpretation: ::reqlang_expr::errors::ExprResult<Value> = $expected_interpretation;
                            ::pretty_assertions::assert_eq!(expected_interpretation, Err(err));
                        }
                    };
                }
            }
        }
    };
}

fn make_test_bytecode(input: Vec<u8>) -> Vec<u8> {
    let mut codes = get_version_bytes().to_vec();
    codes.extend(input);

    codes
}

mod valid {
    use reqlang_expr::{errors::ExprResult, value::Value};

    fn example_builtin(_args: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String("".to_string()))
    }

    test! {
        "foo";

        scenario: identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3))
        ];

        ast should be: Ok(Expr::identifier("foo"));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [
            BuiltinFn {
                name: "foo",
                args: &[],
                return_type: Type::String,
                func: crate::valid::example_builtin
            }.into()
        ];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::USER_BUILTIN, 0
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET USER_BUILTIN    0 == 'foo'\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Fn(BuiltinFn {
                name: "foo",
                args: &[],
                return_type: Type::String,
                func: crate::valid::example_builtin
            }.into()));
    }

    test! {
        "`test string`";

        scenario: string string;

        tokens should be: vec![
            Ok((0, Token::String("test string".to_string()), 13)),
        ];

        ast should be: Ok(Expr::string("test string"));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::CONSTANT, 0
            ]),
            vec!["test string".to_string()],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 CONSTANT            0 == 'test string'\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("test string".to_string()));
    }

    test! {
        "(noop)";

        scenario: call noop;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("noop"), 5)),
            Ok((5, Token::RParan, 6)),
        ];

        ast should be: Ok(Expr::call((Expr::identifier("noop"), 1..5), vec![]));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 1,
                opcode::CALL, 0
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         1 == 'noop'\n0003 CALL             (0 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("noop".to_string()));
    }

    test! {
        "noop";

        scenario: reference builtin;

        tokens should be: vec![
            Ok((0, Token::identifier("noop"), 4))
        ];

        ast should be: Ok(Expr::identifier("noop"));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         1 == 'noop'\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Fn(BuiltinFn {
            name: "noop",
            args: &[],
            return_type: Type::String,
            func: crate::valid::example_builtin
        }.into()));
    }

    test! {
        "(id (noop))";

        scenario: call id with noop call;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("id"), 3)),
            Ok((4, Token::LParan, 5)),
            Ok((5, Token::identifier("noop"), 9)),
            Ok((9, Token::RParan, 10)),
            Ok((10, Token::RParan, 11)),
        ];

        ast should be: Ok(Expr::call((Expr::identifier("id"), 1..3), vec![
            (Expr::call((Expr::identifier("noop"), 5..9), vec![]), 4..10)
        ]));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 0,
                opcode::GET, lookup::BUILTIN, 1,
                opcode::CALL, 0,
                opcode::CALL, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         0 == 'id'\n0003 GET BUILTIN         1 == 'noop'\n0006 CALL             (0 args)\n0008 CALL             (1 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "noop".to_string()));
    }

    test! {
        "(id `test value`)";

        scenario: call id with string;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("id"), 3)),
            Ok((4, Token::String("test value".to_string()), 16)),
            Ok((16, Token::RParan, 17)),
        ];

        ast should be: Ok(Expr::call((Expr::identifier("id"), 1..3), vec![
            (Expr::string("test value"), 4..16)
        ]));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 0,
                opcode::CONSTANT, 0,
                opcode::CALL, 1
            ]),
            vec![
                "test value".to_string(),
            ],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         0 == 'id'\n0003 CONSTANT            0 == 'test value'\n0005 CALL             (1 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "test value".to_string()));
    }

    test! {
        "(id :b)";

        scenario: call id with var;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("id"), 3)),
            Ok((4, Token::identifier(":b"), 6)),
            Ok((6, Token::RParan, 7)),
        ];

        ast should be: Ok(Expr::call((Expr::identifier("id"), 1..3), vec![
            (Expr::identifier_with_type(":b", Type::String), 4..6)
        ]));

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 0,
                opcode::GET, lookup::VAR, 1,
                opcode::CALL, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         0 == 'id'\n0003 GET VAR             1 == 'b'\n0006 CALL             (1 args)\n";

        runtime env: {
            vars: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    test! {
        "(id (id :b))";

        scenario: call id with a call to id;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("id"), 3)),
            Ok((4, Token::LParan, 5)),
            Ok((5, Token::identifier("id"), 7)),
            Ok((8, Token::identifier(":b"), 10)),
            Ok((10, Token::RParan, 11)),
            Ok((11, Token::RParan, 12)),
        ];

        ast should be: Ok(Expr::call((Expr::identifier("id"), 1..3), vec![
            (Expr::call((Expr::identifier("id"), 5..7), vec![
                (Expr::identifier_with_type(":b", Type::String), 8..10)
            ]), 4..11)
        ]));

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 0,

                opcode::GET, lookup::BUILTIN, 0,
                opcode::GET, lookup::VAR, 1,
                opcode::CALL, 1,

                opcode::CALL, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         0 == 'id'\n0003 GET BUILTIN         0 == 'id'\n0006 GET VAR             1 == 'b'\n0009 CALL             (1 args)\n0011 CALL             (1 args)\n";

        runtime env: {
            vars: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }
    test! {
        ":b";

        scenario: variable identifier;

        tokens should be: vec![
            Ok((0, Token::identifier(":b"), 2))
        ];

        ast should be: Ok(Expr::identifier_with_type(":b", Type::String));

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::VAR, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET VAR             1 == 'b'\n";

        runtime env: {
            vars: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    test! {
        "?b";

        scenario: prompt identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("?b"), 2))
        ];

        ast should be: Ok(Expr::identifier_with_type("?b", Type::String));

        env: (vec![], vec!["a".to_string(), "b".to_string()], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::PROMPT, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET PROMPT          1 == 'b'\n";

        runtime env: {
            prompts: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    test! {
        "(id ?b)";

        scenario: call id with prompt;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("id"), 3)),
            Ok((4, Token::identifier("?b"), 6)),
            Ok((6, Token::RParan, 7)),
        ];

        ast should be: Ok(Expr::call((Expr::identifier("id"), 1..3), vec![
            (Expr::identifier_with_type("?b", Type::String), 4..6)
        ]));

        env: (vec![], vec!["a".to_string(), "b".to_string()], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 0,
                opcode::GET, lookup::PROMPT, 1,
                opcode::CALL, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         0 == 'id'\n0003 GET PROMPT          1 == 'b'\n0006 CALL             (1 args)\n";

        runtime env: {
            prompts: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    test! {
        "!b";

        scenario: secret identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("!b"), 2))
        ];

        ast should be: Ok(Expr::identifier_with_type("!b", Type::String));

        env: (vec![], vec![], vec!["a".to_string(), "b".to_string()], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::SECRET, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET SECRET          1 == 'b'\n";

        runtime env: {
            secrets: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    test! {
        "@b";

        scenario: client context identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("@b"), 2))
        ];

        ast should be: Ok(Expr::identifier_with_type("@b", Type::Value));

        env: (vec![], vec![], vec![], vec!["a".to_string(), "b".to_string()]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::CLIENT_CTX, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET CLIENT_CTX      1 == 'b'\n";

        runtime env: {
            client_context: vec![Value::String("a_value".to_string()), Value::String("b_value".to_string())],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    // test! {
    //     "(foo)";

    //     scenario: call without args;

    //     tokens should be: vec![
    //         Ok((0, Token::LParan, 1)),
    //         Ok((1, Token::identifier("foo"), 4)),
    //         Ok((4, Token::RParan, 5))
    //     ];

    //     ast should be: Ok(Expr::call(
    //         (Expr::identifier("foo"), 1..4),
    //         vec![]
    //     ));

    //     env: (vec![], vec![], vec![], vec![]);

    //     user builtins: [BuiltinFn {
    //         name: "foo",
    //         args: vec![],
    //         return_type: Type::String,
    //         func: crate::valid::example_builtin
    //     }.into()];

    //     compiles to: Ok(ExprByteCode::new(
    //         crate::make_test_bytecode(vec![
    //             opcode::GET, lookup::USER_BUILTIN, 0,
    //             opcode::CALL, 0
    //         ]),
    //         vec![]
    //     ));

    //     disassembles to: "VERSION 0700\n----\n0000 GET USER_BUILTIN    0 == 'foo'\n0003 CALL             (0 args)\n";

    //     runtime env: {
    //         ..Default::default()
    //     };

    //     interpets to: Ok(Value::String("".to_string()));
    // }

    // test! {
    //     "(foo :a)";

    //     scenario: call with identifier arg;

    //     tokens should be: vec![
    //         Ok((0, Token::LParan, 1)),
    //         Ok((1, Token::identifier("foo"), 4)),
    //         Ok((5, Token::identifier(":a"), 7)),
    //         Ok((7, Token::RParan, 8))
    //     ];

    //     ast should be: Ok(Expr::call(
    //         (Expr::identifier("foo"), 1..4),
    //         vec![(Expr::identifier_with_type(":a", Type::String), 5..7)]
    //     ));

    //     env: (vec!["a".to_string()], vec![], vec![], vec![]);

    //     user builtins: [BuiltinFn {
    //         name: "foo",
    //         args: &[FnArg::new("value", Type::String)],
    //         return_type: Type::String,
    //         func: crate::valid::example_builtin
    //     }.into()];

    //     compiles to: Ok(ExprByteCode::new(
    //         crate::make_test_bytecode(vec![
    //             opcode::GET, lookup::USER_BUILTIN, 0,
    //             opcode::GET, lookup::VAR, 0,
    //             opcode::CALL, 1
    //         ]),
    //         vec![]
    //     ));

    //     disassembles to: "VERSION 0700\n----\n0000 GET USER_BUILTIN    0 == 'foo'\n0003 GET VAR             0 == 'a'\n0006 CALL             (1 args)\n";

    //     runtime env: {
    //         vars: vec!["a_value".to_string()],
    //         ..Default::default()
    //     };

    //     interpets to: Ok(Value::String("".to_string()));
    // }

    // test! {
    //     "(foo (bar :a) (fiz ?b) (baz !c))";

    //     scenario: call with multiple call args;

    //     tokens should be: vec![
    //         Ok((0, Token::LParan, 1)),
    //         Ok((1, Token::identifier("foo"), 4)),

    //         Ok((5, Token::LParan, 6)),
    //         Ok((6, Token::identifier("bar"), 9)),
    //         Ok((10, Token::identifier(":a"), 12)),
    //         Ok((12, Token::RParan, 13)),

    //         Ok((14, Token::LParan, 15)),
    //         Ok((15, Token::identifier("fiz"), 18)),
    //         Ok((19, Token::identifier("?b"), 21)),
    //         Ok((21, Token::RParan, 22)),

    //         Ok((23, Token::LParan, 24)),
    //         Ok((24, Token::identifier("baz"), 27)),
    //         Ok((28, Token::identifier("!c"), 30)),
    //         Ok((30, Token::RParan, 31)),

    //         Ok((31, Token::RParan, 32))
    //     ];

    //     ast should be: Ok(Expr::call(
    //         (Expr::identifier("foo"), 1..4),
    //         vec![
    //             (Expr::call(
    //                 (Expr::identifier("bar"), 6..9),
    //                 vec![
    //                     (Expr::identifier_with_type(":a", Type::String), 10..12)
    //                 ]
    //             ), 5..13),
    //             (Expr::call(
    //                 (Expr::identifier("fiz"), 15..18),
    //                 vec![(Expr::identifier_with_type("?b", Type::String), 19..21)]
    //             ), 14..22),
    //             (Expr::call(
    //                 (Expr::identifier("baz"), 24..27),
    //                 vec![(Expr::identifier_with_type("!c", Type::String), 28..30)]
    //             ), 23..31)
    //         ]
    //     ));

    //     env: (
    //         vec!["a".to_string()],
    //         vec!["b".to_string()],
    //         vec!["c".to_string()],
    //         vec![]
    //     );

    //     user builtins: [
    //         BuiltinFn {
    //             name: "foo",
    //             args: &[
    //                 FnArg::new("a", Type::String),
    //                 FnArg::new("b", Type::String),
    //                 FnArg::new("c", Type::String)
    //             ],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into(),
    //         BuiltinFn {
    //             name: "bar",
    //             args: &[FnArg::new("value", Type::String)],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into(),
    //         BuiltinFn {
    //             name: "fiz",
    //             args: &[FnArg::new("value", Type::String)],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into(),
    //         BuiltinFn {
    //             name: "baz",
    //             args: &[FnArg::new("value", Type::String)],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into()
    //     ];

    //     compiles to: Ok(ExprByteCode::new(
    //         crate::make_test_bytecode(vec![
    //             opcode::GET, lookup::USER_BUILTIN, 0,
    //             opcode::GET, lookup::USER_BUILTIN, 1,
    //             opcode::GET, lookup::VAR, 0,
    //             opcode::CALL, 1,
    //             opcode::GET, lookup::USER_BUILTIN, 2,
    //             opcode::GET, lookup::PROMPT, 0,
    //             opcode::CALL, 1,
    //             opcode::GET, lookup::USER_BUILTIN, 3,
    //             opcode::GET, lookup::SECRET, 0,
    //             opcode::CALL, 1,
    //             opcode::CALL, 3
    //         ]),
    //         vec![]
    //     ));

    //     disassembles to: "VERSION 0700\n----\n0000 GET USER_BUILTIN    0 == 'foo'\n0003 GET USER_BUILTIN    1 == 'bar'\n0006 GET VAR             0 == 'a'\n0009 CALL             (1 args)\n0011 GET USER_BUILTIN    2 == 'fiz'\n0014 GET PROMPT          0 == 'b'\n0017 CALL             (1 args)\n0019 GET USER_BUILTIN    3 == 'baz'\n0022 GET SECRET          0 == 'c'\n0025 CALL             (1 args)\n0027 CALL             (3 args)\n";

    //     runtime env: {
    //         vars: vec!["a_value".to_string()],
    //         prompts: vec!["b_value".to_string()],
    //         secrets: vec!["c_value".to_string()],
    //         ..Default::default()
    //     };

    //     interpets to: Ok(Value::String("".to_string()));
    // }

    test! {
        "true";

        scenario: boolean true;

        tokens should be: vec![
            Ok((0, Token::True, 4)),
        ];

        ast should be: Ok(Expr::bool(true));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::TRUE
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 TRUE\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "false";

        scenario: boolean false;

        tokens should be: vec![
            Ok((0, Token::False, 5)),
        ];

        ast should be: Ok(Expr::bool(false));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::FALSE
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 FALSE\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(not false)";

        scenario: not;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("not"), 4)),
            Ok((5, Token::False, 10)),
            Ok((10, Token::RParan, 11)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("not"), 1..4).into(),
                args: vec![(
                    Expr::bool(false),
                    5..10
                )]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 16,
                opcode::FALSE,
                opcode::CALL, 1
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN        16 == 'not'\n0003 FALSE\n0004 CALL             (1 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(and true false)";

        scenario: and true false;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("and"), 4)),
            Ok((5, Token::True, 9)),
            Ok((10, Token::False, 15)),
            Ok((15, Token::RParan, 16)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("and"), 1..4).into(),
                args: vec![
                    (Expr::bool(true), 5..9),
                    (Expr::bool(false), 10..15)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 3,
                opcode::TRUE,
                opcode::FALSE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         3 == 'and'\n0003 TRUE\n0004 FALSE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(and true true)";

        scenario: and true true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("and"), 4)),
            Ok((5, Token::True, 9)),
            Ok((10, Token::True, 14)),
            Ok((14, Token::RParan, 15)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("and"), 1..4).into(),
                args: vec![
                    (Expr::bool(true), 5..9),
                    (Expr::bool(true), 10..14)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 3,
                opcode::TRUE,
                opcode::TRUE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         3 == 'and'\n0003 TRUE\n0004 TRUE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(and false true)";

        scenario: and false true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("and"), 4)),
            Ok((5, Token::False, 10)),
            Ok((11, Token::True, 15)),
            Ok((15, Token::RParan, 16)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("and"), 1..4).into(),
                args: vec![
                    (Expr::bool(false), 5..10),
                    (Expr::bool(true), 11..15)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 3,
                opcode::FALSE, opcode::TRUE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         3 == 'and'\n0003 FALSE\n0004 TRUE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(or true false)";

        scenario: or true false;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("or"), 3)),
            Ok((4, Token::True, 8)),
            Ok((9, Token::False, 14)),
            Ok((14, Token::RParan, 15)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("or"), 1..3).into(),
                args: vec![
                    (Expr::bool(true), 4..8),
                    (Expr::bool(false), 9..14)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 4,
                opcode::TRUE, opcode::FALSE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         4 == 'or'\n0003 TRUE\n0004 FALSE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(or true true)";

        scenario: or true true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("or"), 3)),
            Ok((4, Token::True, 8)),
            Ok((9, Token::True, 13)),
            Ok((13, Token::RParan, 14)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("or"), 1..3).into(),
                args: vec![
                    (Expr::bool(true), 4..8),
                    (Expr::bool(true), 9..13)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 4,
                opcode::TRUE, opcode::TRUE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         4 == 'or'\n0003 TRUE\n0004 TRUE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(or false true)";

        scenario: or false true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("or"), 3)),
            Ok((4, Token::False, 9)),
            Ok((10, Token::True, 14)),
            Ok((14, Token::RParan, 15)),
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("or"), 1..3).into(),
                args: vec![
                    (Expr::bool(false), 4..9),
                    (Expr::bool(true), 10..14)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 4,
                opcode::FALSE,
                opcode::TRUE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         4 == 'or'\n0003 FALSE\n0004 TRUE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(cond true `foo` `bar`)";

        scenario: cond true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("cond"), 5)),
            Ok((6, Token::True, 10)),
            Ok((11, Token::String("foo".to_string()), 16)),
            Ok((17, Token::String("bar".to_string()), 22)),
            Ok((22, Token::RParan, 23))
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("cond"), 1..5).into(),
                args: vec![
                    (Expr::bool(true), 6..10),
                    (Expr::string("foo"), 11..16),
                    (Expr::string("bar"), 17..22)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 5,
                opcode::TRUE,
                opcode::CONSTANT, 0,
                opcode::CONSTANT, 1,
                opcode::CALL, 3
            ]),
            vec![
                "foo".to_string(),
                "bar".to_string(),
            ],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         5 == 'cond'\n0003 TRUE\n0004 CONSTANT            0 == 'foo'\n0006 CONSTANT            1 == 'bar'\n0008 CALL             (3 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foo".to_string()));
    }

    test! {
        "(cond false `foo` `bar`)";

        scenario: cond false;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("cond"), 5)),
            Ok((6, Token::False, 11)),
            Ok((12, Token::String("foo".to_string()), 17)),
            Ok((18, Token::String("bar".to_string()), 23)),
            Ok((23, Token::RParan, 24))
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("cond"), 1..5).into(),
                args: vec![
                    (Expr::bool(false), 6..11),
                    (Expr::string("foo"), 12..17),
                    (Expr::string("bar"), 18..23)
                ]
            }.into())
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 5,
                opcode::FALSE,
                opcode::CONSTANT, 0,
                opcode::CONSTANT, 1,
                opcode::CALL, 3
            ]),
            vec![
                "foo".to_string(),
                "bar".to_string(),
            ],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         5 == 'cond'\n0003 FALSE\n0004 CONSTANT            0 == 'foo'\n0006 CONSTANT            1 == 'bar'\n0008 CALL             (3 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("bar".to_string()));
    }

    test! {
        "(is_empty ``)";

        scenario: call is_empty with empty string;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(is_empty `foo`)";

        scenario: call is_empty with non empty string;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(to_str `foo`)";

        scenario: to_str string;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foo".to_string()));
    }

    test! {
        "(to_str true)";

        scenario: to_str bool true;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("true".to_string()));
    }

    test! {
        "(to_str false)";

        scenario: to_str bool false;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("false".to_string()));
    }

    test! {
        "(to_str id)";

        scenario: to_str builtin id;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("id(value: Value) -> Value".to_string()));
    }

    test! {
        "(to_str (id (noop)))";

        scenario: to_str call to builtin id;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("noop".to_string()));
    }

    test! {
        "(concat `foo` `bar`)";

        scenario: concat two strings;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foobar".to_string()));
    }

    test! {
        "(concat `a` `b` `c` `d` `e` `f` `g` `h` `i` `j`)";

        scenario: concat max number of ten args;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("abcdefghij".to_string()));
    }

    test! {
        "(concat `foo` false)";

        scenario: concat string and bool;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foofalse".to_string()));
    }

    test! {
        "(contains `foo` `foobar`)";

        scenario: contains string in string true;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(contains `baz` `foobar`)";

        scenario: contains string in string false;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(contains `foo` :a)";

        scenario: contains variable in string true;

        env: (vec!["a".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            vars: vec!["foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(contains :a `foobar`)";

        scenario: contains string in variable true;

        env: (vec!["a".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            vars: vec!["foo".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(contains :a :b)";

        scenario: contains variable in variable true v2;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("contains"), 9)),
            Ok((10, Token::identifier(":a"), 12)),
            Ok((13, Token::identifier(":b"), 15)),
            Ok((15, Token::RParan, 16))
        ];

        ast should be: Ok(
            Expr::Call(ExprCall {
                callee: (Expr::identifier("contains"), 1..9).into(),
                args: vec![
                    (Expr::Identifier(ExprIdentifier(":a".to_string(), IdentifierKind::Var, Some(Type::String)).into()), 10..12),
                    (Expr::identifier_with_type(":b", Type::String), 13..15)
                ]
            }.into())
        );

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 8,
                opcode::GET, lookup::VAR, 0,
                opcode::GET, lookup::VAR, 1,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN         8 == 'contains'\n0003 GET VAR             0 == 'a'\n0006 GET VAR             1 == 'b'\n0009 CALL             (2 args)\n";

        runtime env: {
            vars: vec!["foo".to_string(), "foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(contains `baz` :a)";

        scenario: contains variable in string false;

        env: (vec!["a".to_string()], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            vars: vec!["foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(contains `foo` ?a)";

        scenario: contains prompt in string true;

        env: (vec![], vec!["a".to_string()], vec![], vec![]);

        user builtins: [];

        runtime env: {
            prompts: vec!["foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(contains `baz` ?a)";

        scenario: contains prompt in string false;

        env: (vec![], vec!["a".to_string()], vec![], vec![]);

        user builtins: [];

        runtime env: {
            prompts: vec!["foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(contains `foo` !a)";

        scenario: contains secret in string true;

        env: (vec![], vec![], vec!["a".to_string()], vec![]);

        user builtins: [];

        runtime env: {
            secrets: vec!["foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(contains `baz` !a)";

        scenario: contains secret in string false;

        env: (vec![], vec![], vec!["a".to_string()], vec![]);

        user builtins: [];

        runtime env: {
            secrets: vec!["foobar".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(trim ` foo `)";

        scenario: trim;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foo".to_string()));
    }

    test! {
        "(trim_start ` foo `)";

        scenario: trim start;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foo ".to_string()));
    }

    test! {
        "(trim_end ` foo `)";

        scenario: trim end;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String(" foo".to_string()));
    }

    test! {
        "(lowercase `FOO`)";

        scenario: lowercase;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foo".to_string()));
    }

    test! {
        "(uppercase `foo`)";

        scenario: uppercase;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("FOO".to_string()));
    }

    test! {
        "(type `foo`)";

        scenario: type string;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(Type::String.into()).into()));
    }

    test! {
        "(type true)";

        scenario: type bool true;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(Type::Bool.into()).into()));
    }

    test! {
        "(type false)";

        scenario: type bool false;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(Type::Bool.into()).into()));
    }

    test! {
        "(type id)";

        scenario: type builtin id;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(
            Value::Type(
                Type::Type(Type::Fn {
                    args: vec![Type::Value],
                    variadic_arg: None,
                    returns: Type::Value.into()
                }.into()).into()
            )
        );
    }

    test! {
        "(type concat)";

        scenario: type builtin concat;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(
            Value::Type(
                Type::Type(Type::Fn {
                    args: vec![Type::Value, Type::Value],
                    variadic_arg: Some(Type::Value.into()),
                    returns: Type::String.into()
                }.into()).into()
            )
        );
    }

    test! {
        "@intest";

        scenario: client in test value;

        tokens should be: vec![
            Ok((0, Token::identifier("@intest"), 7)),
        ];

        ast should be: Ok(
            Expr::identifier_with_type("@intest", Type::Value)
        );

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::CLIENT_CTX, 0
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET CLIENT_CTX      0 == 'intest'\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "((id id) `foo`)";

        scenario: call expr with call expression as calle;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("foo".to_string()));
    }

    test! {
        "(eq true true)";

        scenario: eq true true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("eq"), 3)),
            Ok((4, Token::True, 8)),
            Ok((9, Token::True, 13)),
            Ok((13, Token::RParan, 14))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("eq"), 1..3),
            vec![
                (Expr::bool(true), 4..8),
                (Expr::bool(true), 9..13),
            ]
        ));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 15,
                opcode::TRUE,
                opcode::TRUE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN        15 == 'eq'\n0003 TRUE\n0004 TRUE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(eq false true)";

        scenario: eq false true;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("eq"), 3)),
            Ok((4, Token::False, 9)),
            Ok((10, Token::True, 14)),
            Ok((14, Token::RParan, 15))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("eq"), 1..3),
            vec![
                (Expr::bool(false), 4..9),
                (Expr::bool(true), 10..14),
            ]
        ));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Ok(ExprByteCode::new(
            crate::make_test_bytecode(vec![
                opcode::GET, lookup::BUILTIN, 15,
                opcode::FALSE,
                opcode::TRUE,
                opcode::CALL, 2
            ]),
            vec![],
            vec![]
        ));

        disassembles to: "VERSION 0700\n----\n0000 GET BUILTIN        15 == 'eq'\n0003 FALSE\n0004 TRUE\n0005 CALL             (2 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(eq `foo` `foo`)";

        scenario: builtin eq same string;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(eq `foo` `bar`)";

        scenario: builtin eq different strings;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(eq true true)";

        scenario: builtin eq same bool;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(eq true false)";

        scenario: builtin eq different bool;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "(eq id id)";

        scenario: builtin eq same fn;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(true));
    }

    test! {
        "(eq id concat)";

        scenario: builtin eq different fns;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Bool(false));
    }

    test! {
        "String";

        scenario: type literal String;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(Type::String.into()).into()));
    }

    test! {
        "Bool";

        scenario: type literal Bool;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(Type::Bool.into()).into()));
    }

    test! {
        "Type<String>";

        scenario: type literal Type String;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(Type::String.into()).into()));
    }

    test! {
        "Fn() -> Value";

        scenario: type literal Type Fn no args;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(
            Type::Fn {
                args: vec![],
                variadic_arg: None,
                returns: Type::Value.into()
            }.into()).into()));
    }

    test! {
        "Fn(Value) -> Value";

        scenario: type literal Type Fn arg;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(
            Type::Fn {
                args: vec![Type::Value],
                variadic_arg: None,
                returns: Type::Value.into()
            }.into()).into()));
    }

    test! {
        "Fn(Value, Value) -> Value";

        scenario: type literal Type Fn args;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(
            Type::Fn {
                args: vec![Type::Value, Type::Value],
                variadic_arg: None,
                returns: Type::Value.into()
            }.into()).into()));
    }

    test! {
        "Fn(...Value) -> Value";

        scenario: type literal Type Fn vargs;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(
            Type::Fn {
                args: vec![],
                variadic_arg: Some(Type::Value.into()),
                returns: Type::Value.into()
            }.into()).into()));
    }

    test! {
        "Fn(Value, ...Value) -> Value";

        scenario: type literal Type Fn arg and vargs;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(
            Type::Fn {
                args: vec![Type::Value],
                variadic_arg: Some(Type::Value.into()),
                returns: Type::Value.into()
            }.into()).into()));
    }

    test! {
        "Fn(Value, Value, ...Value) -> Value";

        scenario: type literal Type Fn args and vargs;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Type(Type::Type(
            Type::Fn {
                args: vec![Type::Value, Type::Value],
                variadic_arg: Some(Type::Value.into()),
                returns: Type::Value.into()
            }.into()).into()));
    }

    // test! {
    //     "(foo bar fiz baz)";

    //     scenario: call with multiple identifier args;

    //     tokens should be: vec![
    //         Ok((0, Token::LParan, 1)),
    //         Ok((1, Token::identifier("foo"), 4)),
    //         Ok((5, Token::identifier("bar"), 8)),
    //         Ok((9, Token::identifier("fiz"), 12)),
    //         Ok((13, Token::identifier("baz"), 16)),
    //         Ok((16, Token::RParan, 17))
    //     ];

    //     ast should be: Ok(Expr::call(
    //         (Expr::identifier("foo"), 1..4),
    //         vec![
    //             (Expr::identifier("bar"), 5..8),
    //             (Expr::identifier("fiz"), 9..12),
    //             (Expr::identifier("baz"), 13..16)
    //         ]
    //     ));

    //     env: (vec![], vec![], vec![], vec![]);

    //     user builtins: [
    //         BuiltinFn {
    //             name: "foo",
    //             args: vec![
    //                 FnArg::new("a", Type::Fn { args: vec![], returns: Type::Value.into(), variadic_arg: None }),
    //                 FnArg::new("b", Type::Fn { args: vec![], returns: Type::Value.into(), variadic_arg: None }),
    //                 FnArg::new("c", Type::Fn { args: vec![], returns: Type::Value.into(), variadic_arg: None }),
    //             ],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into(),
    //         BuiltinFn {
    //             name: "bar",
    //             args: vec![],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into(),
    //         BuiltinFn {
    //             name: "fiz",
    //             args: vec![],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into(),
    //         BuiltinFn {
    //             name: "baz",
    //             args: vec![],
    //             return_type: Type::String,
    //             func: crate::valid::example_builtin
    //         }.into()
    //     ];

    //     compiles to: Ok(ExprByteCode::new(
    //         crate::make_test_bytecode(vec![
    //             opcode::GET, lookup::USER_BUILTIN, 0,
    //             opcode::GET, lookup::USER_BUILTIN, 1,
    //             opcode::GET, lookup::USER_BUILTIN, 2,
    //             opcode::GET, lookup::USER_BUILTIN, 3,
    //             opcode::CALL, 3
    //         ]),
    //         vec![]
    //     ));

    //     disassembles to: "VERSION 0700\n----\n0000 GET USER_BUILTIN    0 == 'foo'\n0003 GET USER_BUILTIN    1 == 'bar'\n0006 GET USER_BUILTIN    2 == 'fiz'\n0009 GET USER_BUILTIN    3 == 'baz'\n0012 CALL             (3 args)\n";

    //     runtime env: {
    //         ..Default::default()
    //     };

    //     interpets to: Ok(Value::String("".to_string()));
    // }
}

mod invalid {
    test! {
        "()";

        scenario: unit;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::RParan, 2))
        ];

        ast should be: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from(")"),
                expected: vec![r#""(""#.to_string(), r#""Fn""#.to_string(), r#""true""#.to_string(), r#""false""#.to_string(), "string".to_string(), "identifier".to_string(), "ty".to_string()]
            }.into(),
            1..2
        )]);

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from(")"),
                expected: vec![r#""(""#.to_string(), r#""Fn""#.to_string(), r#""true""#.to_string(), r#""false""#.to_string(), "string".to_string(), "identifier".to_string(), "ty".to_string()]
            }.into(),
            1..2
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from(")"),
                expected: vec![r#""(""#.to_string(), r#""Fn""#.to_string(), r#""true""#.to_string(), r#""false""#.to_string(), "string".to_string(), "identifier".to_string(), "ty".to_string()]
            }.into(),
            1..2
        )]);
    }

    test! {
        ".foo";

        scenario: identifier starting with invalid character;

        tokens should be: vec![
            Err((LexicalError::InvalidToken.into(), 0..0)),
            Ok((1, Token::identifier("foo"), 4))
        ];

        ast should be: Err(vec![(
            LexicalError::InvalidToken.into(),
            0..0
        )]);

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            LexicalError::InvalidToken.into(),
            0..0
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            LexicalError::InvalidToken.into(),
            0..0
        )]);
    }

    test! {
        "foo!bar";

        scenario: identifier starting with invalid character prefixed with identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3)),
            Ok((3, Token::identifier("!bar"), 7)),
        ];

        ast should be: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from("!bar"),
                expected: vec![]
            }.into(),
            3..7
        )]);

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from("!bar"),
                expected: vec![]
            }.into(),
            3..7
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from("!bar"),
                expected: vec![]
            }.into(),
            3..7
        )]);
    }

    test! {
        "foo bar";

        scenario: multiple identifiers outside a call;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3)),
            Ok((4, Token::identifier("bar"), 7)),
        ];

        ast should be: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from("bar"),
                expected: vec![]
            }.into(),
            4..7
        )]);

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from("bar"),
                expected: vec![]
            }.into(),
            4..7
        )]);


        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            SyntaxError::UnrecognizedToken {
                token: String::from("bar"),
                expected: vec![]
            }.into(),
            4..7
        )]);
    }

    test! {
        "`test string";

        scenario: unterminated string;

        tokens should be: vec![
            Err((LexicalError::InvalidToken.into(), 0..0)),
        ];

        ast should be: Err(vec![(
            LexicalError::InvalidToken.into(),
            0..0
        )]);

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            LexicalError::InvalidToken.into(),
            0..0
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            LexicalError::InvalidToken.into(),
            0..0
        )]);
    }

    test! {
        "foo";

        scenario: undefined identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3)),
        ];

        ast should be: Ok(Expr::Identifier(ExprIdentifier::new("foo").into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            CompileError::Undefined("foo".to_string()).into(),
            0..3
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            CompileError::Undefined("foo".to_string()).into(),
            0..3
        )]);
    }

    test! {
        "(concat foo foo)";

        scenario: undefined identifier multiple;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("concat"), 7)),
            Ok((8, Token::identifier("foo"), 11)),
            Ok((12, Token::identifier("foo"), 15)),
            Ok((15, Token::RParan, 16)),
        ];

        ast should be: Ok(Expr::Call(ExprCall {
            callee: (Expr::identifier("concat"), 1..7).into(),
            args: vec![
                (
                    Expr::identifier("foo"),
                    8..11
                ),
                (
                    Expr::identifier("foo"),
                    12..15
                )
            ]
        }.into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![
            (CompileError::Undefined("foo".to_string()).into(), 8..11),
            (CompileError::Undefined("foo".to_string()).into(), 12..15)
        ]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![
            (CompileError::Undefined("foo".to_string()).into(), 8..11),
            (CompileError::Undefined("foo".to_string()).into(), 12..15)
        ]);
    }

    test! {
        "(to_str (id (noop))";

        scenario: missing end parans;

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            SyntaxError::UnrecognizedEOF {
                expected: vec![r#""(""#.to_string(), r#"")""#.to_string(), r#""Fn""#.to_string(), r#""true""#.to_string(), r#""false""#.to_string(), "string".to_string(), "identifier".to_string(), "ty".to_string()]
            }.into(),
            19..19
        )]);
    }

    test! {
        "(not)";

        scenario: not called with zero args;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("not"), 4)),
            Ok((4, Token::RParan, 5)),
        ];

        ast should be: Ok(Expr::Call(ExprCall {
            callee: (Expr::identifier("not"), 1..4).into(),
            args: vec![]
        }.into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            CompileError::WrongNumberOfArgs { expected: 1, actual: 0 }.into(),
            0..5
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            CompileError::WrongNumberOfArgs { expected: 1, actual: 0 }.into(),
            0..5
        )]);
    }

    test! {
        "(not true false)";

        scenario: not called with to many args;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("not"), 4)),
            Ok((5, Token::True, 9)),
            Ok((10, Token::False, 15)),
            Ok((15, Token::RParan, 16)),
        ];

        ast should be: Ok(Expr::Call(ExprCall {
            callee: (Expr::identifier("not"), 1..4).into(),
            args: vec![
                (Expr::bool(true), 5..9),
                (Expr::bool(false), 10..15),
            ]
        }.into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            CompileError::WrongNumberOfArgs { expected: 1, actual: 2 }.into(),
            0..16
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            CompileError::WrongNumberOfArgs { expected: 1, actual: 2 }.into(),
            0..16
        )]);
    }

    test! {
        "(not `true`)";

        scenario: not called with string arg;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("not"), 4)),
            Ok((5, Token::String(String::from("true")), 11)),
            Ok((11, Token::RParan, 12)),
        ];

        ast should be: Ok(Expr::Call(ExprCall {
            callee: (Expr::identifier("not"), 1..4).into(),
            args: vec![
                (Expr::string("true"), 5..11),
            ]
        }.into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            CompileError::TypeMismatch { expected: Type::Bool, actual: Type::String }.into(),
            5..11
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            CompileError::TypeMismatch { expected: Type::Bool, actual: Type::String }.into(),
            5..11
        )]);
    }

    test! {
        "(not true `true`)";

        scenario: not called with bool then a string arg;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("not"), 4)),
            Ok((5, Token::True, 9)),
            Ok((10, Token::String(String::from("true")), 16)),
            Ok((16, Token::RParan, 17)),
        ];

        ast should be: Ok(Expr::Call(ExprCall {
            callee: (Expr::identifier("not"), 1..4).into(),
            args: vec![
                (Expr::bool(true), 5..9),
                (Expr::string("true"), 10..16),
            ]
        }.into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![(
            CompileError::WrongNumberOfArgs { expected: 1, actual: 2 }.into(),
            0..17
        )]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![(
            CompileError::WrongNumberOfArgs { expected: 1, actual: 2 }.into(),
            0..17
        )]);
    }

    test! {
        "(not `true` true)";

        scenario: not called with string then a bool arg;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("not"), 4)),
            Ok((5, Token::String(String::from("true")), 11)),
            Ok((12, Token::True, 16)),
            Ok((16, Token::RParan, 17)),
        ];

        ast should be: Ok(Expr::Call(ExprCall {
            callee: (Expr::identifier("not"), 1..4).into(),
            args: vec![
                (Expr::string("true"), 5..11),
                (Expr::bool(true), 12..16),
            ]
        }.into()));

        env: (vec![], vec![], vec![], vec![]);

        user builtins: [];

        compiles to: Err(vec![
            (CompileError::WrongNumberOfArgs { expected: 1, actual: 2 }.into(), 0..17),
            (CompileError::TypeMismatch { expected: Type::Bool, actual: Type::String }.into(), 5..11)
        ]);

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(vec![
            (CompileError::WrongNumberOfArgs { expected: 1, actual: 2 }.into(), 0..17),
            (CompileError::TypeMismatch { expected: Type::Bool, actual: Type::String }.into(), 5..11)
        ]);
    }
}
