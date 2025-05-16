macro_rules! test {
    (
        $source:expr;
        scenario: $test_name:ident $( $test_name2:ident)*;
        tokens should be: $expected_tokens:expr;
        ast should be: $expected_ast:expr;
        compiles to: $expected_op_codes:expr;
    ) => {
        ::pastey::paste! {
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
                let tokens = ::reqlang_expr::lexer::Lexer::new($source);
                let ast = ::reqlang_expr::exprlang::ExprParser::new().parse(tokens);

                if let Ok(ast) = ast {
                    let op_codes = ::reqlang_expr::compiler::compile(&ast);
                    let expected_op_codes: Vec<u8> = $expected_op_codes;

                    ::pretty_assertions::assert_eq!(expected_op_codes, op_codes);
                }
            }
        }
    };
}

mod valid {
    use reqlang_expr::prelude::*;

    test! {
        "foo";

        scenario: identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("foo"), 3))
        ];

        ast should be: Ok(Expr::identifier("foo"));

        compiles to: vec![opcode::BUILTIN, 0];
    }

    test! {
        ":variable";

        scenario: variable identifier;

        tokens should be: vec![
            Ok((0, Token::identifier(":variable"), 9))
        ];

        ast should be: Ok(Expr::identifier(":variable"));

        compiles to: vec![opcode::VAR, 0];
    }

    test! {
        "?prompt";

        scenario: prompt identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("?prompt"), 7))
        ];

        ast should be: Ok(Expr::identifier("?prompt"));

        compiles to: vec![opcode::PROMPT, 0];
    }

    test! {
        "!secret";

        scenario: secret identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("!secret"), 7))
        ];

        ast should be: Ok(Expr::identifier("!secret"));

        compiles to: vec![opcode::SECRET, 0];
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

        compiles to: vec![opcode::CALL, opcode::BUILTIN, 0, 0];
    }

    test! {
        "(foo bar)";

        scenario: call with identifier arg;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("foo"), 4)),
            Ok((5, Token::identifier("bar"), 8)),
            Ok((8, Token::RParan, 9))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("foo"), 1..4),
            vec![(Expr::identifier("bar"), 5..8)]
        ));

        compiles to: vec![opcode::CALL, opcode::BUILTIN, 0, 1, opcode::BUILTIN, 1];
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

        compiles to: vec![
            opcode::CALL,
            opcode::BUILTIN,
            0,
            3,
            opcode::BUILTIN,
            1,
            opcode::BUILTIN,
            2,
            opcode::BUILTIN,
            3
        ];
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

        compiles to: vec![
            opcode::CALL,
            opcode::BUILTIN,
            0,
            3,
            opcode::CALL,
            opcode::BUILTIN,
            1,
            1,
            opcode::VAR,
            0,
            opcode::CALL,
            opcode::BUILTIN,
            2,
            1,
            opcode::PROMPT,
            0,
            opcode::CALL,
            opcode::BUILTIN,
            3,
            1,
            opcode::SECRET,
            0,
        ];
    }
}

mod invalid {
    use reqlang_expr::prelude::*;

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

        compiles to: vec![];
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

        compiles to: vec![];
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

        compiles to: vec![];
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

        compiles to: vec![];
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

        compiles to: vec![];
    }
}
