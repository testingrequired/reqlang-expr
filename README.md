# Reqlang Expression Language

A small (tiny) WIP expression language for [reqlang](https://github.com/testingrequired/reqlang)'s templating engine.

## Install

![Crates.io Version](https://img.shields.io/crates/v/reqlang-expr)

```toml
[dependencies]
reqlang-expr = "0.5.0"
```

```sh
cargo add reqlang-expr
```

## Usage

See [USAGE.md](./USAGE.md) and [examples](./examples/) for usage examples.

## Project

[![Verify](https://github.com/testingrequired/reqlang-expr/actions/workflows/ci.yml/badge.svg)](https://github.com/testingrequired/reqlang-expr/actions/workflows/ci.yml)

- [Lexer](./src/lexer.rs)
- [Parser](./src/parser.rs), [Grammar](./src/grammer.lalrpop), [AST](./src/ast.rs)
- [Bytecode Compiler](./src/compiler.rs)
- [VM interpreter](./src/vm.rs)
- [Disassembler](./src/disassembler.rs)
- [Types](./src/types.rs)
- [REPL](#repl)
- [Example Usage](./examples/)
- [Specification Examples](./spec/)
- [Tests](./tests/integration_tests.rs)

## Syntax

The syntax is s-expression like. There are only [builtin functions](#builtin-functions), identifiers and string literals.

| Syntax      | Description                                 |
| ----------- | ------------------------------------------- |
| `:a`        | Reference to the variable `a`               |
| `?b`        | Reference to the prompt `b`                 |
| `!c`        | Reference to the secret `c`                 |
| `id`        | Reference to the builtin `id`               |
| `@key`      | Reference to the client context value `key` |
| `(id :a)`   | Call to builtin `id` with arguments: `:a`   |
| `` `foo` `` | String literal                              |
| `true`      | Literal boolean value `true`                |
| `false`     | Literal boolean value `false`               |

See [/spec](./spec/) for more syntax examples.

### Builtin Functions

| Fn                                                        | Description                                     |
| --------------------------------------------------------- | ----------------------------------------------- |
| `id(value: Value) -> Value`                               | Returns the string arugment passed to it        |
| `noop() -> String`                                        | Returns the string "noop"                       |
| `is_empty(value: String) -> String`                       | Checks if the given string is empty             |
| `and(a: Bool, b: Bool) -> Bool`                           | Logical AND operation between two booleans      |
| `or(a: Bool, b: Bool) -> Bool`                            | Logical OR operation between two booleans       |
| `cond(cond: Bool, then: Value, else: Value) -> Bool`      | Conditional expression                          |
| `to_str(value: Value) -> String`                          | Converts a value to its string representation   |
| `concat(a: String, b: String, ...rest: String) -> String` | Concatenates a list of values in to a string    |
| `contains(needle: String, haystack: String) -> Bool`      | Checks for a substring match                    |
| `trim(value: String) -> String`                           | Trim whitespace from a string                   |
| `trim_start(value: String) -> String`                     | Trim whitespace from the start of a string      |
| `trim_end(value: String) -> String`                       | Trim whitespace from the end of a string        |
| `lowercase(value: String) -> String`                      | Return a lowercase version of a string          |
| `uppercase(value: String) -> String`                      | Return a uppercase version of a string          |
| `eq(a: Value, b: Value) -> Bool`                          | Compare two values for equality                 |
| `type(value: Value) -> String`                            | Get the string representation of a value's type |

### Operator Functions

| Fn                         | Description                              |
| -------------------------- | ---------------------------------------- |
| `not(value: Bool) -> Bool` | Logical NOT operation on a boolean value |

### Why Backticks For Strings?

These expressions will be embedded in places where double quotes are common (e.g. JSON). Single quotes weren't chosen due to their use in prose e.g. weren't

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
cargo run -q --example disassembler -- spec/call_id_with_id_arg.expr
```

#### stderr

```
0000 GET BUILTIN         0 == 'id'
0003 GET BUILTIN         0 == 'id'
0006 GET BUILTIN         1 == 'noop'
0009 CALL             (0 args)
0011 CALL             (1 args)
0013 CALL             (1 args)
```

### Interpreter

Interpret an expression.

```sh
cargo run -q --example interpreter -- spec/variable.expr \
    --vars b=b_value
```

#### stdout

```
`b_value`
```

### REPL

A simple REPL to interpret expressions.

```sh
cargo run -q --example repl
```

#### Repl Mode

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

#### Reference Last String Value

The last returned string value can be referenced using `@_`.

```
interpret   > `value`

interpret   > @_

`value`

interpret   > (id @_)

`value`

interpret   > id

builtin id(1)

interpret   > (@_ `value`)

`value`
```

#### Set Variable

```
interpret   > /set var key = value

interpret   > :key

`value`
```

#### Set Prompt

```
interpret   > /set prompt key = value

interpret   > ?key

`value`
```

#### Set Secret

```
interpret   > /set secret key = value

interpret   > !key

`value`
```

#### Set Client Context

```
interpret   > /set client key = value

interpret   > @key

`value`
```

#### Print Current Environment

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

#### Exit

```
interpret   > /exit
```

#### Example

```
interpret   > (id :foo)

`bar`
```
