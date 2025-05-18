# Reqlang Expression Language

A small (tiny) WIP expression language for [reqlang](https://github.com/testingrequired/reqlang)'s templating engine.

| Syntax         | Description                                        |
| -------------- | -------------------------------------------------- |
| `:a`           | Reference to the variable `a`                      |
| `?b`           | Reference to the prompt `b`                        |
| `?c`           | Reference to the secret `c`                        |
| `f`            | Reference to the function `f`                      |
| `(f :a ?b !c)` | Function call `f` with arguments: `:a`, `?b`, `!c` |

- [Lexer](./src/lexer.rs)
- [Parser](./src/exprlang.lalrpop), [AST](./src/ast.rs)
- [Bytecode Compiler](./src/compiler.rs)
- [VM interpreter](./src/vm.rs)
- [CLI](./src/main.rs)
- [Examples](./spec/)
- [Tests](./tests/integration_tests.rs)

## Built With

- [lalrpop](https://github.com/lalrpop/lalrpop)
- [logos](https://github.com/maciejhirsz/logos)
