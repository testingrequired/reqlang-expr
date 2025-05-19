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

### Parser

Parse an expression into an AST.

```sh
cargo run -q --example parser spec/call_with_args.expr
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

### Disassembler

Compile expression and disassemble it.

```sh
cargo run -q --example disassembler -- spec/call_with_args.expr \
    --builtins fn_name:3 \
    --vars a \
    --prompts b \
    --secrets c
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
