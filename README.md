# Reqlang Expression Language

A small (tiny) WIP expression language for [reqlang](https://github.com/testingrequired/reqlang)'s templating engine.

- [Lexer](./src/lexer.rs)
- [Parser](./src/exprlang.lalrpop), [AST](./src/ast.rs)
- [Bytecode Compiler](./src/compiler.rs)
- [VM interpreter](./src/vm.rs)
- [Tests](./tests/integration_tests.rs)

## Built With

- [lalrpop](https://github.com/lalrpop/lalrpop)
- [logos](https://github.com/maciejhirsz/logos)
