macro_rules! test {
    (
        $source:expr;
        scenario: $test_name:ident $( $test_name2:ident)*;
        tokens should be: $expected_tokens:expr;
        ast should be: $expected_ast:expr;
        env: $env:tt;
        builtins: $builtins:tt;
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
                    let tokens = ::reqlang_expr::lexer::Lexer::new($source).collect::<Vec<_>>();

                    ::pretty_assertions::assert_eq!($expected_tokens, tokens);
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _ast >]() {
                    let tokens = ::reqlang_expr::lexer::Lexer::new($source);
                    let ast = ::reqlang_expr::exprlang::ExprParser::new().parse(tokens);

                    ::pretty_assertions::assert_eq!($expected_ast, ast);
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _op_codes >]() {
                    let mut env: Env = Env::new$env;

                    env.add_builtins(vec!$builtins);

                    let tokens = ::reqlang_expr::lexer::Lexer::new($source);
                    let ast = ::reqlang_expr::exprlang::ExprParser::new().parse(tokens);

                    if let Ok(ast) = ast {
                        let op_codes = ::reqlang_expr::compiler::compile(&ast, &env);
                        let expected_op_codes: Vec<u8> = $expected_op_codes;

                        ::pretty_assertions::assert_eq!(expected_op_codes, op_codes.codes);
                    }
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _op_codes_disassemble_to >]() {
                    let mut env: Env = ::reqlang_expr::compiler::Env::new$env;

                    env.add_builtins(vec!$builtins);

                    let tokens = ::reqlang_expr::lexer::Lexer::new($source);
                    let ast = ::reqlang_expr::exprlang::ExprParser::new().parse(tokens);

                    if let Ok(ast) = ast {
                        let op_codes = ::reqlang_expr::compiler::compile(&ast, &env);
                        let expected_disassembly: String = $expected_disassembly.to_string();
                        let disassemble = ::reqlang_expr::disassembler::Disassembler::new(&op_codes, &env);
                        let disassembly = disassemble.disassemble(None);

                        ::pretty_assertions::assert_eq!(expected_disassembly, disassembly);
                    }
                }

                #[test]
                fn [< $test_name:lower $(_ $test_name2:lower)* _interprets_without_error >]() {
                    let mut env: Env = Env::new$env;

                    env.add_builtins(vec!$builtins);

                    let tokens = ::reqlang_expr::lexer::Lexer::new($source);
                    let ast = ::reqlang_expr::exprlang::ExprParser::new().parse(tokens);

                    if let Ok(ast) = ast {
                        let op_codes = ::reqlang_expr::compiler::compile(&ast, &env);

                        let mut vm = Vm::new();
                        let runtime_env: RuntimeEnv = RuntimeEnv$runtime_env;

                        let value = vm.interpret(op_codes.into(), &env, &runtime_env);

                        let expected_interpretation: Result<Value, ()> = $expected_interpretation;

                        ::pretty_assertions::assert_eq!(expected_interpretation, value);
                    }
                }
            }
        }
    };
}

mod valid {
    test! {
        "`test string`";

        scenario: string string;

        tokens should be: vec![
            Ok((0, Token::String("test string".to_string()), 13)),
        ];

        ast should be: Ok(Expr::string("test string"));

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![
            opcode::CONSTANT, 0
        ];

        disassembles to: "0000 CONSTANT            0 == 'test string'\n";

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

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![
            opcode::GET, lookup::BUILTIN, 1,
            opcode::CALL, 0
        ];

        disassembles to: "0000 GET                 1 == 'noop'\n0003 CALL             (0 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("noop".to_string()));
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

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![
            opcode::GET, lookup::BUILTIN, 0,
            opcode::GET, lookup::BUILTIN, 1,
            opcode::CALL, 0,
            opcode::CALL, 1
        ];

        disassembles to: "0000 GET                 0 == 'id'\n0003 GET                 1 == 'noop'\n0006 CALL             (0 args)\n0008 CALL             (1 args)\n";

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

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![
            opcode::GET, lookup::BUILTIN, 0,
            opcode::CONSTANT, 0,
            opcode::CALL, 1
        ];

        disassembles to: "0000 GET                 0 == 'id'\n0003 CONSTANT            0 == 'test value'\n0005 CALL             (1 args)\n";

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
            (Expr::identifier(":b"), 4..6)
        ]));

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![]);

        builtins: [];

        compiles to: vec![opcode::GET, lookup::BUILTIN, 0, opcode::GET, lookup::VAR, 1, opcode::CALL, 1];

        disassembles to: "0000 GET                 0 == 'id'\n0003 GET                 1 == 'b'\n0006 CALL             (1 args)\n";

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
                (Expr::identifier(":b"), 8..10)
            ]), 4..11)
        ]));

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![]);

        builtins: [];

        compiles to: vec![
            opcode::GET, lookup::BUILTIN, 0,

            opcode::GET, lookup::BUILTIN, 0,
            opcode::GET, lookup::VAR, 1,
            opcode::CALL, 1,

            opcode::CALL, 1
        ];

        disassembles to: "0000 GET                 0 == 'id'\n0003 GET                 0 == 'id'\n0006 GET                 1 == 'b'\n0009 CALL             (1 args)\n0011 CALL             (1 args)\n";

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

        ast should be: Ok(Expr::identifier(":b"));

        env: (vec!["a".to_string(), "b".to_string()], vec![], vec![]);

        builtins: [];

        compiles to: vec![opcode::GET, lookup::VAR, 1];

        disassembles to: "0000 GET                 1 == 'b'\n";

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

        ast should be: Ok(Expr::identifier("?b"));

        env: (vec![], vec!["a".to_string(), "b".to_string()], vec![]);

        builtins: [];

        compiles to: vec![opcode::GET, lookup::PROMPT, 1];

        disassembles to: "0000 GET                 1 == 'b'\n";

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
            (Expr::identifier("?b"), 4..6)
        ]));

        env: (vec![], vec!["a".to_string(), "b".to_string()], vec![]);

        builtins: [];

        compiles to: vec![opcode::GET, lookup::BUILTIN, 0, opcode::GET, lookup::PROMPT, 1, opcode::CALL, 1];

        disassembles to: "0000 GET                 0 == 'id'\n0003 GET                 1 == 'b'\n0006 CALL             (1 args)\n";

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

        ast should be: Ok(Expr::identifier("!b"));

        env: (vec![], vec![], vec!["a".to_string(), "b".to_string()]);

        builtins: [];

        compiles to: vec![opcode::GET, lookup::SECRET, 1];

        disassembles to: "0000 GET                 1 == 'b'\n";

        runtime env: {
            secrets: vec!["a_value".to_string(), "b_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String(
            "b_value".to_string()));
    }

    test! {
        "(foo)";

        scenario: call without args;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("foo"), 4)),
            Ok((4, Token::RParan, 5))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("foo"), 1..4),
            vec![]
        ));

        env: (vec![], vec![], vec![]);

        builtins: [BuiltinFn {
            name: "foo".to_string(),
            arity: 0,
            func: std::rc::Rc::new(|_| Value::String(String::new()))
        }.into()];

        compiles to: vec![opcode::GET, lookup::BUILTIN, 2, opcode::CALL, 0];

        disassembles to: "0000 GET                 2 == 'foo'\n0003 CALL             (0 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "(foo :a)";

        scenario: call with identifier arg;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("foo"), 4)),
            Ok((5, Token::identifier(":a"), 7)),
            Ok((7, Token::RParan, 8))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("foo"), 1..4),
            vec![(Expr::identifier(":a"), 5..7)]
        ));

        env: (vec!["a".to_string()], vec![], vec![]);

        builtins: [BuiltinFn {
            name: "foo".to_string(),
            arity: 0,
            func: std::rc::Rc::new(|_| Value::String(String::new()))
        }.into()];

        compiles to: vec![opcode::GET, lookup::BUILTIN, 2, opcode::GET, lookup::VAR, 0, opcode::CALL, 1];

        disassembles to: "0000 GET                 2 == 'foo'\n0003 GET                 0 == 'a'\n0006 CALL             (1 args)\n";

        runtime env: {
            vars: vec!["a_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "(foo (bar :a) (fiz ?b) (baz !c))";

        scenario: call with multiple call args;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("foo"), 4)),

            Ok((5, Token::LParan, 6)),
            Ok((6, Token::identifier("bar"), 9)),
            Ok((10, Token::identifier(":a"), 12)),
            Ok((12, Token::RParan, 13)),

            Ok((14, Token::LParan, 15)),
            Ok((15, Token::identifier("fiz"), 18)),
            Ok((19, Token::identifier("?b"), 21)),
            Ok((21, Token::RParan, 22)),

            Ok((23, Token::LParan, 24)),
            Ok((24, Token::identifier("baz"), 27)),
            Ok((28, Token::identifier("!c"), 30)),
            Ok((30, Token::RParan, 31)),

            Ok((31, Token::RParan, 32))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("foo"), 1..4),
            vec![
                (Expr::call(
                    (Expr::identifier("bar"), 6..9),
                    vec![
                        (Expr::identifier(":a"), 10..12)
                    ]
                ), 5..13),
                (Expr::call(
                    (Expr::identifier("fiz"), 15..18),
                    vec![(Expr::identifier("?b"), 19..21)]
                ), 14..22),
                (Expr::call(
                    (Expr::identifier("baz"), 24..27),
                    vec![(Expr::identifier("!c"), 28..30)]
                ), 23..31)
            ]
        ));

        env: (
            vec!["a".to_string()],
            vec!["b".to_string()],
            vec!["c".to_string()]
        );

        builtins: [
            BuiltinFn {
                name: "foo".to_string(),
                arity: 3,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into(),
            BuiltinFn {
                name: "bar".to_string(),
                arity: 1,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into(),
            BuiltinFn {
                name: "fiz".to_string(),
                arity: 1,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into(),
            BuiltinFn {
                name: "baz".to_string(),
                arity: 1,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into()
        ];

        compiles to: vec![
            opcode::GET,
            lookup::BUILTIN, // foo
            2,

            opcode::GET,
            lookup::BUILTIN, // bar
            3,
            opcode::GET,
            lookup::VAR, // :a
            0,
            opcode::CALL, // bar
            1,

            opcode::GET,
            lookup::BUILTIN, // fiz
            4,
            opcode::GET,
            lookup::PROMPT, // ?b
            0,
            opcode::CALL, // fiz
            1,

            opcode::GET,
            lookup::BUILTIN, // baz
            5,
            opcode::GET,
            lookup::SECRET, // !c
            0,
            opcode::CALL, // baz
            1,

            opcode::CALL, // foo
            3
        ];

        disassembles to: "0000 GET                 2 == 'foo'\n0003 GET                 3 == 'bar'\n0006 GET                 0 == 'a'\n0009 CALL             (1 args)\n0011 GET                 4 == 'fiz'\n0014 GET                 0 == 'b'\n0017 CALL             (1 args)\n0019 GET                 5 == 'baz'\n0022 GET                 0 == 'c'\n0025 CALL             (1 args)\n0027 CALL             (3 args)\n";

        runtime env: {
            vars: vec!["a_value".to_string()],
            prompts: vec!["b_value".to_string()],
            secrets: vec!["c_value".to_string()],
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }
}

mod invalid {
    test! {
        "foo";

        scenario: identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3))
        ];

        ast should be: Ok(Expr::identifier("foo"));

        env: (vec![], vec![], vec![]);

        builtins: [
            BuiltinFn {
                name: "foo".to_string(),
                arity: 0,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into()
        ];

        compiles to: vec![opcode::GET, lookup::BUILTIN, 2];

        disassembles to: "0000 GET                 2 == 'foo'\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::Fn(BuiltinFn {
                name: "foo".to_string(),
                arity: 0,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into()));
    }

    test! {
        "(foo bar fiz baz)";

        scenario: call with multiple identifier args;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("foo"), 4)),
            Ok((5, Token::identifier("bar"), 8)),
            Ok((9, Token::identifier("fiz"), 12)),
            Ok((13, Token::identifier("baz"), 16)),
            Ok((16, Token::RParan, 17))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("foo"), 1..4),
            vec![
                (Expr::identifier("bar"), 5..8),
                (Expr::identifier("fiz"), 9..12),
                (Expr::identifier("baz"), 13..16)
            ]
        ));

        env: (vec![], vec![], vec![]);

        builtins: [
            BuiltinFn {
                name: "foo".to_string(),
                arity: 3,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into(),
            BuiltinFn {
                name: "bar".to_string(),
                arity: 0,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into(),
            BuiltinFn {
                name: "fiz".to_string(),
                arity: 0,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into(),
            BuiltinFn {
                name: "baz".to_string(),
                arity: 0,
                func: std::rc::Rc::new(|_| Value::String(String::new()))
            }.into()
        ];

        compiles to: vec![
            opcode::GET,
            lookup::BUILTIN,
            2,

            opcode::GET,
            lookup::BUILTIN,
            3,

            opcode::GET,
            lookup::BUILTIN,
            4,

            opcode::GET,
            lookup::BUILTIN,
            5,

            opcode::CALL,
            3
        ];

        disassembles to: "0000 GET                 2 == 'foo'\n0003 GET                 3 == 'bar'\n0006 GET                 4 == 'fiz'\n0009 GET                 5 == 'baz'\n0012 CALL             (3 args)\n";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "()";

        scenario: unit;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::RParan, 2))
        ];

        ast should be: Err(lalrpop_util::ParseError::UnrecognizedToken {
            token: (1, Token::RParan, 2),
            expected: vec!["identifier".to_string()]
        });

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![];

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "((foo) bar)";

        scenario: call using call callee;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::LParan, 2)),
            Ok((2, Token::identifier("foo"), 5)),
            Ok((5, Token::RParan, 6)),
            Ok((7, Token::identifier("bar"), 10)),
            Ok((10, Token::RParan, 11)),
        ];

        ast should be: Err(lalrpop_util::ParseError::UnrecognizedToken {
            token: (1, Token::LParan, 2),
            expected: vec!["identifier".to_string()]
        });

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![];

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        ".foo";

        scenario: identifier starting with invalid character;

        tokens should be: vec![
            Err((LexicalError::InvalidToken, 0..0)),
            Ok((1, Token::identifier("foo"), 4))
        ];

        ast should be: Err(lalrpop_util::ParseError::User {
            error: (LexicalError::InvalidToken, 0..0)
        });

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![];

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "foo!bar";

        scenario: identifier starting with invalid character prefixed with identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3)),
            Ok((3, Token::identifier("!bar"), 7)),
        ];

        ast should be: Err(lalrpop_util::ParseError::UnrecognizedToken {
            token: (3, Token::identifier("!bar"), 7),
            expected: vec![]
        });

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![];

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "foo bar";

        scenario: multiple identifiers outside a call;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3)),
            Ok((4, Token::identifier("bar"), 7)),
        ];

        ast should be: Err(lalrpop_util::ParseError::UnrecognizedToken {
            token: (4, Token::identifier("bar"), 7),
            expected: vec![]
        });

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![];

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Ok(Value::String("".to_string()));
    }

    test! {
        "`test string";

        scenario: unterminated string;

        tokens should be: vec![
            Err((LexicalError::InvalidToken, 0..0)),
        ];

        ast should be: Err(lalrpop_util::ParseError::User {
            error: (LexicalError::InvalidToken, 0..0)
        });

        env: (vec![], vec![], vec![]);

        builtins: [];

        compiles to: vec![
            opcode::CONSTANT, 0
        ];

        disassembles to: "";

        runtime env: {
            ..Default::default()
        };

        interpets to: Err(());
    }
}
