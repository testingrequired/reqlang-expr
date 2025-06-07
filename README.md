# Reqlang Expression Language

A small (tiny) WIP expression language for [reqlang](https://github.com/testingrequired/reqlang)'s templating engine.

## Syntax

The syntax is s-expression like. There are only [builtin functions](#builtin-functions), identifiers and string literals.

| Syntax      | Description                               |
| ----------- | ----------------------------------------- |
| `:a`        | Reference to the variable `a`             |
| `?b`        | Reference to the prompt `b`               |
| `!c`        | Reference to the secret `c`               |
| `id`        | Reference to the builtin `id`             |
| `(id :a)`   | Call to builtin `id` with arguments: `:a` |
| `` `foo` `` | String literal                            |

### Why Backticks For Strings?

These expressions will be embedded in places where double quotes are common (e.g. JSON). Single quotes weren't chosen due to their use in prose e.g. weren't

### Builtin Functions

| Name   | Arity | Description                              | Usage         |
| ------ | ----: | ---------------------------------------- | ------------- |
| `id`   |     1 | Returns the string arugment passed to it | `(id (noop))` |
| `noop` |     0 | Returns the string "noop"                | `(noop)`      |

## Project

[![Verify](https://github.com/testingrequired/reqlang-expr/actions/workflows/ci.yml/badge.svg)](https://github.com/testingrequired/reqlang-expr/actions/workflows/ci.yml)

- [Lexer](./src/lexer.rs)
- [Parser](./src/exprlang.lalrpop), [AST](./src/ast.rs)
- [Bytecode Compiler](./src/compiler.rs)
- [VM interpreter](./src/vm.rs)
- [Disassembler](./src/disassembler.rs)
- [CLI](./src/main.rs)
- [REPL](#repl)
- [Example Usage](./examples/)
- [Specification Examples](./spec/)
- [Tests](./tests/integration_tests.rs)

A more detailed technical breakdown can be found [here](./TECH.md).

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
                "id",
            ),
            3,
        ),
    ),
    Ok(
        (
            4,
            LParan,
            5,
        ),
    ),
    Ok(
        (
            5,
            Identifier(
                "id",
            ),
            7,
        ),
    ),
    Ok(
        (
            8,
            Identifier(
                ":a",
            ),
            10,
        ),
    ),
    Ok(
        (
            10,
            RParan,
            11,
        ),
    ),
    Ok(
        (
            11,
            RParan,
            12,
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
                    "id",
                ),
            ),
            1..3,
        ),
        args: [
            (
                Call(
                    ExprCall {
                        callee: (
                            Identifier(
                                ExprIdentifier(
                                    "id",
                                ),
                            ),
                            5..7,
                        ),
                        args: [
                            (
                                Identifier(
                                    ExprIdentifier(
                                        ":a",
                                    ),
                                ),
                                8..10,
                            ),
                        ],
                    },
                ),
                4..11,
            ),
        ],
    },
)
```

### Compiler

Compile an expression into bytecode to stdout.

```sh
cargo run -q --example compiler -- spec/call_with_args.expr \
    --vars a \
    > output.exprbin
```

#### stderr

```
ExprByteCode {
    codes: [
        1,
        0,
        0,
        1,
        0,
        0,
        1,
        1,
        0,
        0,
        1,
        0,
        1,
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
    --vars a=a_value
```

#### stderr

```
0000 GET                 0 == 'id'
0003 GET                 0 == 'id'
0006 CALL             (1 args)
0008 CALL             (1 args)
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

### Interpreter

Interpret an expression.

```sh
cargo run -q --example interpreter -- spec/call_with_args.expr \
    --vars a=a_value
```

#### stdout

```
String(
    "a_value",
)
```

### REPL

A simple REPL to interpret expressions.

```sh
cargo run -q --example repl
```

#### Commands

##### Repl Mode

The REPL works in different modes:

1. Interpret: Fully interpret an expression using the VM. This is the default.
2. Compile: Compile an expression into its bytecode
3. Disassemble: Compile and disassemble an expression
4. Parse: Parse an expression into an AST
5. Lex: Lex an expression into tokens

```
interpret   > /mode

Current Mode: Interpret

interpret   > /mode compile

compile     > /mode disassemble

disassemble > /mode parse

parse       > /mode lex

lex         > /mode interpret

interpret   > /mode

Current Mode: Interpret
```

##### Set Variable

```
interpret   > /set var key = value

interpret   > :key

value
```

##### Set Prompt

```
interpret   > /set prompt key = value

interpret   > ?key

value
```

##### Set Secret

```
interpret   > /set secret key = value

interpret   > !key

value
```

##### Print Current Environment

```
interpret   > /env

Env {
    builtins: [
        BuiltinFn {
            name: "id",
            arity: 1,
        },
        BuiltinFn {
            name: "noop",
            arity: 0,
        },
    ],
    vars: [],
    prompts: [],
    secrets: [],
}

> /set var isActive = true

> /env

Env {
    builtins: [
        BuiltinFn {
            name: "id",
            arity: 1,
        },
        BuiltinFn {
            name: "noop",
            arity: 0,
        },
    ],
    vars: ["isActive"],
    prompts: [],
    secrets: [],
}
```

#### Example

```
interpret   > (id :foo)

bar
```
