# Technical Breakdown

## Lexer

The lexer takes an input string and returns a stream of tokens.

| Token        | Pattern                       | Description                                                           |
| ------------ | ----------------------------- | --------------------------------------------------------------------- |
| `LParan`     | `(`                           | Opening parantheses for a call expression                             |
| `RParan`     | `)`                           | Closing parantheses for a call expression                             |
| `Identifier` | `[!?:]?[a-zA-Z][a-zA-Z0-9_]*` | Identifer referencing a builtin function, variable, prompt, or secret |
| `String`     | `` `[^`]*` ``                 | A literal string of text delimited by backticks                       |

See: [lexer.rs](./src/lexer.rs)

## Parser

The [parser] takes a stream of tokens from the lexer and constructs an AST (Abstract Syntax Tree).

| Expression   | Description                                                                           |
| ------------ | ------------------------------------------------------------------------------------- |
| `Call`       | A call to a builtin (referenced by identifier) with N expressions passed as arguments |
| `Identifier` | An identifier referencing a builtin, variable, prompt, or secret                      |
| `String`     | A string literal of text                                                              |

See: [exprlang.lalrpop](./src/exprlang.lalrpop), [ast.rs](./src/ast.rs)

## Compiler

The compiler takes an AST from the parser and converts to bytecode.

### Op Codes

| Op Code    | Byte | Args                                | Description                                                 |
| ---------- | ---: | ----------------------------------- | ----------------------------------------------------------- |
| `CALL`     |    0 | $INDEX, $ARG_COUNT                  | Call a function with N arguments.                           |
| `GET`      |    1 | [$LOOK_TYPE](#lookup-types), $INDEX | Get a builtin/variable/prompt/secret from the env by index. |
| `CONSTANT` |    2 | $INDEX                              | Get constant value by index.                                |

#### Lookup Types

| Type      | Lookup Index |
| --------- | -----------: |
| `BUILTIN` |            0 |
| `VAR`     |            1 |
| `PROMPT`  |            2 |
| `SECRET`  |            3 |

See: [compiler.rs](./src/compiler.rs)

## Virtual Machine

The virtual machine (VM) reads through the stream of bytecode, pushing and popping values to a stack, interpreting the input expression.

See: [vm.rs](./src/vm.rs)
