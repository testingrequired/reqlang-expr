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
| `Comma`      | `,`                           | Separator between arguments in `Fn` types                             |
| `LAngle`     | `<`                           | Generic type delimitor                                                |
| `RAngle`     | `>`                           | Generic type delimitor                                                |
| `Arrow`      | `->`                          | Separator between `Fn` arg parans and return type                     |
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

The parser takes a stream of tokens from the lexer and constructs an AST (Abstract Syntax Tree) in the form of a tree of `Expr` nodes.

### Usage

```rust
let source = "(eq (type `Hello`) (type `World`))";

let ast: Expr = parse(&source)?;
```

### Expr

All values in the language are parsed in to an expression: `Expr`.

```rust
pub enum Expr {
    Bool(Box<ExprBool>),
    Identifier(Box<ExprIdentifier>),
    Call(Box<ExprCall>),
    String(Box<ExprString>),
    Error,
}
```

#### Error

Expressions that can't be parsed are represented in the AST as an `Expr::Error`.

#### Span Information

Expressions that contain sub expressions (e.g. `ExprCall`) store those subexpressions and their location in the source as `(Expr, Range<usize>)`.

The expression `(and true false)` would parse to this.

```rust
Expr::Call(ExprCall {
    callee: (Expr::identifier("and"), 1..4).into(),
    args: vec![
        (Expr::bool(true), 5..9),
        (Expr::bool(false), 10..15)
    ]
}.into());
```

Span information `1..4`, `5..9`, `10..15` is stored next to the subexpressions `and`, `true`, and `false`.

#### ExprCall

A call to a builtin (referenced by identifier) with N expressions passed as arguments.

- `` (contains `foo`, `foobar`) ``
- `` (eq `foo` (trim ` foo `)) ``

```rust
pub struct ExprCall {
    pub callee: Box<ExprS>,
    pub args: Vec<ExprS>,
}
```

#### ExprIdentifier

An identifier referencing a builtin, type, variable, prompt, secret, or client value.

- `builtin_name`
- `TypeName`, `TypeName<Type>`, `Fn(Type, ...Type) -> Type`
- `:var_name`
- `?prompt_name`
- `!secret_name`
- `@client_value`

```rust
pub struct ExprIdentifier(pub String, pub IdentifierKind, pub Option<Type>);

pub enum IdentifierKind {
    Builtin,
    Var,
    Prompt,
    Secret,
    Client,
    Type,
}

let ident = ExprIdentifier::new(":foo");
let expr = Expr::Identifier(Box::new(ident));
assert_eq!(IdentifierKind::Var, *ident.identifier_kind());
```

##### Full Name

The full name of the identifier, including the sigil e.g. `:`, `?`, `!`, `@` (for non builtins).

```rust
let ident = ExprIdentifier::new(":foo");
assert_eq!(":foo", ident.full_name());
```

##### Look Up Name

The full name of the identifier, minus the sigil. This is used when looking up identifiers in the compilation and runtime phase.

```rust
let ident = ExprIdentifier::new(":foo");
assert_eq!("foo", ident.lookup_name());
```

##### Identifier Type

The type of the identifier is added in two passes:

1. Variables, prompts, secrets, and client values during the parsing phase
2. Builtins during the compilation phase

#### ExprString

A string literal of text.

- `` `Hello World!` ``

```rust
pub struct ExprString(pub String);
```

#### ExprBool

A boolean literal.

- `true`
- `false`

```rust
pub struct ExprBool(pub bool);
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
    Type(Box<Type>),
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

| Op Code    | Op Byte | Args                             | Description                                                   |
| ---------- | ------: | -------------------------------- | ------------------------------------------------------------- |
| `CALL`     |       0 | $INDEX, $ARG_COUNT               | Call builtin `$INDEX` with `$ARG_COUNT` arguments             |
| `GET`      |       1 | [$LOOKUP](#lookup-types), $INDEX | Get a builtin/variable/prompt/secret from the env by `$INDEX` |
| `CONSTANT` |       2 | $CONST_INDEX                     | Get constant value by `$CONST_INDEX`                          |
| `TRUE`     |       3 |                                  | Push a `true` value on to the stack                           |
| `FALSE`    |       4 |                                  | Push a `false` value on to the stack                          |

### Lookup Types

| Type           | Lookup Index | Description                                          |
| -------------- | -----------: | ---------------------------------------------------- |
| `BUILTIN`      |            0 | Builtin function                                     |
| `VAR`          |            1 | Variable (identifier prefixed with `:`)              |
| `PROMPT`       |            2 | Prompt (identifier prefixed with `?`)                |
| `SECRET`       |            3 | Secret (identifier prefixed with `!`)                |
| `USER_BUILTIN` |            4 | User provided builtin function                       |
| `CLIENT_CTX`   |            5 | Client provided value (identifier prefixed with `@`) |
| `TYPE`         |            6 | A type stored in `ExprByteCode`                      |

### Compile Time Environment

The compiler's environment contains a lists of names for [builtin functions](#builtin-functions), variables, prompts, and secrets.

```rust
pub struct CompileTimeEnv {
    builtins: Vec<BuiltinFn<'static>>,
    user_builtins: Vec<BuiltinFn<'static>>,
    vars: Vec<String>,
    prompts: Vec<String>,
    secrets: Vec<String>,
    client_context: Vec<String>,
}
```

#### Builtin Functions

Builtins are functions provided by the compiler/VM and the only functions available.

```rust
pub struct BuiltinFn<'a> {
    /// Needs to follow identifier naming rules
    pub name: &'static str,
    /// Arguments the function expects
    pub args: &'a [FnArg],
    /// Type returned by the function
    pub return_type: Type,
    /// Function used at runtime
    pub func: fn(Vec<Value>) -> ExprResult<Value>,
}

pub struct FnArg {
    name: String,
    ty: Type,
    varadic: bool,
}
```

See: [builtins.rs](./src/builtins.rs), [types.rs](./src/types.rs), [value.rs](./src/value.rs)

### ExprByteCode

The result of an expression compilation is `ExprByteCode`.

```rust
pub struct ExprByteCode {
    version: [u8; 4],
    codes: Vec<u8>,
    strings: Vec<String>,
    types: Vec<Type>,
}
```

#### Version

The version of the compiler encoded as bytes.

#### Codes

The actual bytecode values.

#### Strings

An indexed list of strings encountered during compilation. These string indexes are referenced by the `opcode::CONSTANT` opcode.

#### Type

An indexed list of types encountered during compilation. These type indexes are referenced by the `opcode::GET` opcode and `lookup::TYPE` lookup.

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
