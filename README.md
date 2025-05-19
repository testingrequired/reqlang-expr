# Reqlang Expression Language

A small (tiny) WIP expression language for [reqlang](https://github.com/testingrequired/reqlang)'s templating engine.

| Syntax         | Description                                        |
| -------------- | -------------------------------------------------- |
| `:a`           | Reference to the variable `a`                      |
| `?b`           | Reference to the prompt `b`                        |
| `!c`           | Reference to the secret `c`                        |
| `f`            | Reference to the function `f`                      |
| `(f :a ?b !c)` | Function call `f` with arguments: `:a`, `?b`, `!c` |

## Project

[![Verify](https://github.com/testingrequired/reqlang-expr/actions/workflows/ci.yml/badge.svg)](https://github.com/testingrequired/reqlang-expr/actions/workflows/ci.yml)

- [Lexer](./src/lexer.rs)
- [Parser](./src/exprlang.lalrpop), [AST](./src/ast.rs)
- [Bytecode Compiler](./src/compiler.rs)
- [VM interpreter](./src/vm.rs)
- [Disassembler](./src/disassembler.rs)
- [CLI](./src/main.rs)
- [Example Usage](./examples/)
- [Specification Examples](./spec/)
- [Tests](./tests/integration_tests.rs)

## Built With

- [lalrpop](https://github.com/lalrpop/lalrpop)
- [logos](https://github.com/maciejhirsz/logos)

## Running Examples

### Lexer

Lex an expression in to a list of tokens.

```sh
cargo run -q --example lexer spec/call_with_args.expr
```

#### stderr

```
[
    Ok(
        (
            0,
            LParan,
            1,
        ),
    ),
    Ok(
        (
            1,
            Identifier(
                "fn_name",
            ),
            8,
        ),
    ),
    Ok(
        (
            9,
            Identifier(
                ":a",
            ),
            11,
        ),
    ),
    Ok(
        (
            12,
            Identifier(
                "?b",
            ),
            14,
        ),
    ),
    Ok(
        (
            15,
            Identifier(
                "!c",
            ),
            17,
        ),
    ),
    Ok(
        (
            17,
            RParan,
            18,
        ),
    ),
]
```

### Parser

Parse an expression into an AST.

```sh
cargo run -q --example parser spec/call_with_args.expr
```

#### stderr

```
Call(
    ExprCall {
        callee: (
            Identifier(
                ExprIdentifier(
                    "fn_name",
                ),
            ),
            1..8,
        ),
        args: [
            (
                Identifier(
                    ExprIdentifier(
                        ":a",
                    ),
                ),
                9..11,
            ),
            (
                Identifier(
                    ExprIdentifier(
                        "?b",
                    ),
                ),
                12..14,
            ),
            (
                Identifier(
                    ExprIdentifier(
                        "!c",
                    ),
                ),
                15..17,
            ),
        ],
    },
)
```

### Compiler

Compile an expression into bytecode to stdout.

```sh
cargo run -q --example compiler -- spec/call_with_args.expr \
    --builtins fn_name:3 \
    --vars a \
    --prompts b \
    --secrets c \
    > output.exprbin
```

### Interpreter

Interpret an expression.

```sh
cargo run -q --example interpreter -- spec/call_with_args.expr \
    --builtins fn_name:3 \
    --vars a \
    --prompts b \
    --secrets c
```

#### stderr

```
ExprByteCode {
    codes: [
        0,
        1,
        0,
        3,
        2,
        0,
        3,
        0,
        4,
        0,
    ],
}
```

#### stdout

```
...BYTECODE...
```

### Disassembler

Compile expression and disassemble it.

```sh
cargo run -q --example disassembler -- spec/call_with_args.expr \
    --builtins fn_name:3 \
    --vars a \
    --prompts b \
    --secrets c
```

#### stderr

```
0000 CALL                0 == fn_name (3 args)
0004 VAR                 0 == 'a'
0006 PROMPT              0 == 'b'
0008 SECRET              0 == 'c'
```

### Disassembler From Bytecode

Read in bytecode from binary file and disassemble it.

```sh
cargo run -q --example disassembler_from_bytecode -- output.exprbin \
    --builtins fn_name:3 \
    --vars a \
    --prompts b \
    --secrets c
```

#### stderr

```
0000 CALL                0 == fn_name (3 args)
0004 VAR                 0 == 'a'
0006 PROMPT              0 == 'b'
0008 SECRET              0 == 'c'
```
