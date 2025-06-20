# Usage

## Prelude

```rust
use reqlang_expr::prelude::*;
```

The prelude provides everything needed to lex, parse, compile, disassemble, and interpret expressions.

## Error Handling: ExprError & ExprResult<T>

`ExprResult<T>` (type alias for `Result<T, Vec<ExprErrorS>>`) is the result type used to handle errors throughout the process of expression evaluation.

- Multiple errors are returned at a time
- Errors have span information

See: [errors.rs](./src/errors.rs), [span.rs](./src/span.rs)

## Lexer

The lexer takes an input string and returns a stream of tokens.

| Token        | Pattern                       | Description                                                           |
| ------------ | ----------------------------- | --------------------------------------------------------------------- |
| `LParan`     | `(`                           | Opening parantheses for a call expression                             |
| `RParan`     | `)`                           | Closing parantheses for a call expression                             |
| `Identifier` | `[!?:]?[a-zA-Z][a-zA-Z0-9_]*` | Identifer referencing a builtin function, variable, prompt, or secret |
| `String`     | `` `[^`]*` ``                 | A literal string of text delimited by backticks                       |
| `True`       | `true`                        | A literal boolean value of `true`                                     |
| `False`      | `false`                       | A literal boolean value of `true`                                     |

### Usage

```rust
let source = "(noop)";
let tokens: lex(&source);
```

See: [lexer.rs](./src/lexer.rs)

## Parser

The parser takes a stream of tokens from the lexer and constructs an AST (Abstract Syntax Tree).

| Expression   | Description                                                                           |
| ------------ | ------------------------------------------------------------------------------------- |
| `Call`       | A call to a builtin (referenced by identifier) with N expressions passed as arguments |
| `Identifier` | An identifier referencing a builtin, variable, prompt, or secret                      |
| `String`     | A string literal of text                                                              |
| `Bool`       | A string literal of text                                                              |

### Usage

```rust
let source = "(noop)";

let ast: Expr = parse(&source)?;
```

See: [parser.rs](./src/parser.rs), [grammar.lalrpop](./src/grammar.lalrpop), [ast.rs](./src/ast.rs)

## Types

```rust
pub enum Type {
    Value,
    String,
    Fn {
        args: Vec<Type>,
        variadic_arg: Option<Box<Type>>,
        returns: Box<Type>,
    },
    Bool,
    Unknown,
}
```

See: [types.rs](./src/types.rs)

## Compiler

The compiler produces bytecode from an AST and a [compile time environment](#compile-time-environment).

### Bytecode

The compiler produces bytecode from an AST in the form of vector of [op codes](#op-codes).

#### Version Byte Prefix

Valid bytecode will always begin with 4 bytes representing the current language version. This means op codes start at ip/index 4.

### Op Codes

| Op Code    | Op Byte | Args                             | Description                                                            |
| ---------- | ------: | -------------------------------- | ---------------------------------------------------------------------- |
| `CALL`     |       0 | $INDEX, $ARG_COUNT               | Call builtin `$INDEX` with `$ARG_COUNT` arguments                      |
| `GET`      |       1 | [$LOOKUP](#lookup-types), $INDEX | Get a builtin/variable/prompt/secret from the env by `$INDEX`          |
| `CONSTANT` |       2 | $CONST_INDEX                     | Get constant value by `$CONST_INDEX`                                   |
| `TRUE`     |       3 |                                  | Push a `true` value on to the stack                                    |
| `FALSE`    |       4 |                                  | Push a `false` value on to the stack                                   |
| `NOT`      |       5 |                                  | Pop a boolean value from the stack, negate, then push it on the stack  |
| `EQ`       |       6 |                                  | Pop last two values from the stack, compare them, push result on stack |
| `TYPE`     |       7 |                                  | Pop the last value from the stack and put it's type on the stack       |

### Lookup Types

| Type           | Lookup Index | Description                                          |
| -------------- | -----------: | ---------------------------------------------------- |
| `BUILTIN`      |            0 | Builtin function                                     |
| `VAR`          |            1 | Variable (identifier prefixed with `:`)              |
| `PROMPT`       |            2 | Prompt (identifier prefixed with `?`)                |
| `SECRET`       |            3 | Secret (identifier prefixed with `!`)                |
| `USER_BUILTIN` |            4 | User provided builtin function                       |
| `CLIENT_CTX`   |            5 | Client provided value (identifier prefixed with `@`) |

### Compile Time Environment

The compiler's environment contains a lists of names for [builtin functions](#builtin-functions), variables, prompts, and secrets.

```rust
pub struct CompileTimeEnv {
    builtins: Vec<Rc<BuiltinFn>>,
    user_builtins: Vec<Rc<BuiltinFn>>,
    vars: Vec<String>,
    prompts: Vec<String>,
    secrets: Vec<String>,
    client_context: Vec<String>,
}
```

#### Builtin Functions

Builtins are functions provided by the compiler/VM and the only functions available.

```rust
pub struct BuiltinFn {
    // Needs to follow identifier naming rules
    pub name: String,
    // Arguments the function expects
    pub args: Vec<FnArg>,
    pub return_type: Type,
    // Function used at runtime
    pub func: std::rc::Rc<dyn Fn(Vec<Value>) -> Value>,
}

pub struct FnArg {
    name: String,
    ty: Type,
    varadic: bool,
}
```

See: [builtins.rs](./src/builtins.rs), [types.rs](./src/types.rs), [value.rs](./src/value.rs)

### Usage

```rust
let source = "(noop)";

let ast: Expr = parse(&source)?;

let var_names = vec![];
let prompt_names = vec![];
let secret_names = vec![];
let client_context_names = vec![];

let mut env = CompileTimeEnv::new(var_names, prompt_names, secret_names, client_context_namess);

let bytecode = compile(&ast, &env)?;
```

See: [compiler.rs](./src/compiler.rs)

## Virtual Machine

The virtual machine (VM) takes in a runtime environment, evaluates a stream of bytecode and produces a [value](#values).

### Values

`Value` represent expression values during runtime.

```rust
pub enum Value {
    String(String),
    Fn(Rc<BuiltinFn>),
    Bool(bool),
    Type(Box<Type>),
}
```

See: [value.rs](./src/value.rs), [builtins.rs](./src/builtins.rs)

#### Convert Values To Types

A `Value`'s `Type` can be retrieved using `get_type()`:

```rust
let value = Value::String("Hello World".to_string());
let value_type: Type = value.get_type();
```

This also works:

```rust
let value = Value::String("Hello World".to_string());
let value_type: Type = value.into();
```

### Runtime Environment

The VM's runtime environment contains a lists of values for variables, prompts, and secrets.

```rust
pub struct RuntimeEnv {
    pub vars: Vec<String>,
    pub prompts: Vec<String>,
    pub secrets: Vec<String>,
    pub client_context: Vec<Value>,
}
```

See: [vm.rs](./src/vm.rs), [value.rs](./src/value.rs)

### Usage

```rust
let source = "(noop)";

let ast: Expr = parse(&source)?;

let var_names = vec![];
let prompt_names = vec![];
let secret_names = vec![];
let client_context_names = vec![];

let mut env = CompileTimeEnv::new(var_names, prompt_names, secret_names, client_context_names);

let bytecode = compile(&ast, &env)?;

let mut vm = Vm::new();

let var_values = vec![];
let prompt_values = vec![];
let secret_values = vec![];
let client_context_values = vec![];

let runtime_env: RuntimeEnv = RuntimeEnv {
    vars: var_values,
    prompts: prompt_values,
    secrets: secret_values,
    client_context_values
};

let value = vm.interpret(bytecode.into(), &env, &runtime_env)?;
```

See: [vm.rs](./src/vm.rs), [value.rs](./src/value.rs)
