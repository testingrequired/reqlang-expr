macro_rules! test {
    (
        $source:expr;
        scenario: $test_name:ident $( $test_name2:ident)*;
        tokens should be: $expected_tokens:expr;
        ast should be: $expected_ast:expr;
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
    }

    test! {
        ":foo";

        scenario: variable reference identifier;

        tokens should be: vec![
            Ok((0, Token::identifier(":foo"), 4))
        ];

        ast should be: Ok(Expr::identifier(":foo"));
    }

    test! {
        "?foo";

        scenario: prompt reference identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("?foo"), 4))
        ];

        ast should be: Ok(Expr::identifier("?foo"));
    }

    test! {
        "!foo";

        scenario: secret reference identifier;

        tokens should be: vec![
            Ok((0, Token::identifier("!foo"), 4))
        ];

        ast should be: Ok(Expr::identifier("!foo"));
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
    }

    test! {
        "(foo (bar) (fiz baz))";

        scenario: call with multiple call args;

        tokens should be: vec![
            Ok((0, Token::LParan, 1)),
            Ok((1, Token::identifier("foo"), 4)),
            Ok((5, Token::LParan, 6)),
            Ok((6, Token::identifier("bar"), 9)),
            Ok((9, Token::RParan, 10)),
            Ok((11, Token::LParan, 12)),
            Ok((12, Token::identifier("fiz"), 15)),
            Ok((16, Token::identifier("baz"), 19)),
            Ok((19, Token::RParan, 20)),
            Ok((20, Token::RParan, 21))
        ];

        ast should be: Ok(Expr::call(
            (Expr::identifier("foo"), 1..4),
            vec![
                (Expr::call(
                    (Expr::identifier("bar"), 6..9),
                    vec![]
                ), 5..10),
                (Expr::call(
                    (Expr::identifier("fiz"), 12..15),
                    vec![(Expr::identifier("baz"), 16..19)]
                ), 11..20)
            ]
        ));
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
    }
}
