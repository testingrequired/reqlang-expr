//! The compiler and associated types

use core::fmt;
use std::{fmt::Display, ops::Range, rc::Rc};

use crate::{
    ast::Expr,
    errors::{ExprError, ExprResult, TypeError::WrongNumberOfArgs},
    vm::Value,
};

pub mod opcode {
    iota::iota! {
        pub const
        CALL: u8 = iota;,
        GET,
        CONSTANT,
        TRUE,
        FALSE
    }
}

/// Types of lookups for the GET op code
///
/// Used at compile time to encode lookup indexes
///
/// Used at runtime to use lookup indexes to reference runtime values
pub mod lookup {
    iota::iota! {
        pub const
        BUILTIN: u8 = iota;,
        VAR,
        PROMPT,
        SECRET,
        USER_BUILTIN,
        CLIENT_CTX
    }
}

/// Try to get a string from a list
fn get(list: &[String], identifier: &str) -> Option<u8> {
    list.iter().position(|x| x == identifier).map(|i| i as u8)
}

/// Builtin function used in expressions
pub struct BuiltinFn {
    // Needs to follow identifier naming rules
    pub name: String,
    // Number of arguments the function expects
    pub arity: FnArity,
    // Function used at runtime
    pub func: Rc<dyn Fn(Vec<Value>) -> Value>,
}

impl BuiltinFn {
    pub fn arity(&self) -> u8 {
        match self.arity {
            FnArity::N(n) => n,
            FnArity::Variadic { n } => n,
        }
    }

    pub fn arity_matches(&self, arity: u8) -> bool {
        match self.arity {
            FnArity::N(n) => n == arity,
            FnArity::Variadic { n } => n <= arity,
        }
    }
}

impl PartialEq for BuiltinFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "builtin {}({})", self.name, self.arity)
    }
}

#[derive(Debug, PartialEq)]
pub enum FnArity {
    N(u8),
    Variadic { n: u8 },
}

impl Display for FnArity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            FnArity::N(n) => n.to_string(),
            FnArity::Variadic { n } => {
                if *n == 0 {
                    "...".to_string()
                } else {
                    format!("{n}, ...")
                }
            }
        };

        write!(f, "{}", string)
    }
}

pub struct BuiltinFns;

impl BuiltinFns {
    pub fn id(args: Vec<Value>) -> Value {
        let arg = args.first().unwrap();

        arg.get_string().into()
    }

    pub fn noop(_: Vec<Value>) -> Value {
        Value::String(String::from("noop"))
    }

    pub fn is_empty(args: Vec<Value>) -> Value {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string();

        Value::Bool(string_arg.is_empty())
    }

    pub fn not(args: Vec<Value>) -> Value {
        let bool_arg = args
            .first()
            .expect("should have boolean expression passed")
            .get_bool();

        Value::Bool(!bool_arg)
    }

    pub fn and(args: Vec<Value>) -> Value {
        let a_arg = args
            .first()
            .expect("should have first expression passed")
            .get_bool();
        let b_arg = args
            .get(1)
            .expect("should have second expression passed")
            .get_bool();

        Value::Bool(a_arg && b_arg)
    }

    pub fn or(args: Vec<Value>) -> Value {
        let a_arg = args
            .first()
            .expect("should have first expression passed")
            .get_bool();
        let b_arg = args
            .get(1)
            .expect("should have second expression passed")
            .get_bool();

        Value::Bool(a_arg || b_arg)
    }

    pub fn cond(args: Vec<Value>) -> Value {
        let cond_arg = args
            .first()
            .expect("should have cond expression passed")
            .get_bool();
        let then_arg = args
            .get(1)
            .cloned()
            .expect("should have then expression passed");
        let else_arg = args
            .get(2)
            .cloned()
            .expect("should have else expression passed");

        if cond_arg { then_arg } else { else_arg }
    }

    pub fn to_str(args: Vec<Value>) -> Value {
        let value_arg = args.first().expect("should have string expression passed");

        match value_arg {
            Value::String(_) => value_arg.clone(),
            _ => Value::String(value_arg.to_string()),
        }
    }

    pub fn concat(args: Vec<Value>) -> Value {
        let mut result = String::new();

        for arg in args {
            let value = match arg {
                Value::String(string) => string,
                _ => arg.to_string(),
            };

            result.push_str(value.as_str());
        }

        Value::String(result)
    }

    pub fn contains(args: Vec<Value>) -> Value {
        let needle_arg = args
            .first()
            .expect("should have first expression passed")
            .get_string();
        let haystack_arg = args
            .get(1)
            .expect("should have second expression passed")
            .get_string();

        Value::Bool(haystack_arg.contains(needle_arg))
    }

    pub fn trim(args: Vec<Value>) -> Value {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string();

        Value::String(string_arg.trim().to_string())
    }

    pub fn trim_start(args: Vec<Value>) -> Value {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string();

        Value::String(string_arg.trim_start().to_string())
    }

    pub fn trim_end(args: Vec<Value>) -> Value {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string();

        Value::String(string_arg.trim_end().to_string())
    }

    pub fn lowercase(args: Vec<Value>) -> Value {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string();

        Value::String(string_arg.to_lowercase().to_string())
    }

    pub fn uppercase(args: Vec<Value>) -> Value {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string();

        Value::String(string_arg.to_uppercase().to_string())
    }

    pub fn eq(args: Vec<Value>) -> Value {
        let a_arg = args.first().expect("should have first expression passed");
        let b_arg = args.get(1).expect("should have second expression passed");

        Value::Bool(a_arg == b_arg)
    }
}

#[derive(Debug)]
pub struct CompileTimeEnv {
    builtins: Vec<Rc<BuiltinFn>>,
    user_builtins: Vec<Rc<BuiltinFn>>,
    vars: Vec<String>,
    prompts: Vec<String>,
    secrets: Vec<String>,
    client_context: Vec<String>,
}

impl Default for CompileTimeEnv {
    fn default() -> Self {
        Self {
            builtins: vec![
                Rc::new(BuiltinFn {
                    name: String::from("id"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::id),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("noop"),
                    arity: FnArity::N(0),
                    func: Rc::new(BuiltinFns::noop),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("is_empty"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::is_empty),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("not"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::not),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("and"),
                    arity: FnArity::N(2),
                    func: Rc::new(BuiltinFns::and),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("or"),
                    arity: FnArity::N(2),
                    func: Rc::new(BuiltinFns::or),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("cond"),
                    arity: FnArity::N(3),
                    func: Rc::new(BuiltinFns::cond),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("to_str"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::to_str),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("concat"),
                    arity: FnArity::Variadic { n: 0 },
                    func: Rc::new(BuiltinFns::concat),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("contains"),
                    arity: FnArity::N(2),
                    func: Rc::new(BuiltinFns::contains),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("trim"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::trim),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("trim_start"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::trim_start),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("trim_end"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::trim_end),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("lowercase"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::lowercase),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("uppercase"),
                    arity: FnArity::N(1),
                    func: Rc::new(BuiltinFns::uppercase),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("eq"),
                    arity: FnArity::N(2),
                    func: Rc::new(BuiltinFns::eq),
                }),
            ],
            user_builtins: vec![],
            vars: Vec::new(),
            prompts: Vec::new(),
            secrets: Vec::new(),
            client_context: Vec::new(),
        }
    }
}

impl CompileTimeEnv {
    pub fn new(
        vars: Vec<String>,
        prompts: Vec<String>,
        secrets: Vec<String>,
        client_context: Vec<String>,
    ) -> Self {
        Self {
            vars,
            prompts,
            secrets,
            client_context,
            ..Default::default()
        }
    }

    pub fn get_builtin_index(&self, name: &str) -> Option<(&Rc<BuiltinFn>, u8)> {
        let index = self.builtins.iter().position(|x| x.name == name);

        let result = index.map(|i| (self.builtins.get(i).unwrap(), i as u8));
        result
    }

    pub fn get_user_builtin_index(&self, name: &str) -> Option<(&Rc<BuiltinFn>, u8)> {
        let index = self.user_builtins.iter().position(|x| x.name == name);

        let result = index.map(|i| (self.user_builtins.get(i).unwrap(), i as u8));
        result
    }

    pub fn add_user_builtins(&mut self, builtins: Vec<Rc<BuiltinFn>>) {
        for builtin in builtins {
            self.add_user_builtin(builtin);
        }
    }

    pub fn add_user_builtin(&mut self, builtin: Rc<BuiltinFn>) {
        self.user_builtins.push(builtin);
    }

    pub fn get_builtin(&self, index: usize) -> Option<&Rc<BuiltinFn>> {
        self.builtins.get(index)
    }

    pub fn get_user_builtin(&self, index: usize) -> Option<&Rc<BuiltinFn>> {
        self.user_builtins.get(index)
    }

    pub fn get_var(&self, index: usize) -> Option<&String> {
        self.vars.get(index)
    }

    pub fn get_prompt(&self, index: usize) -> Option<&String> {
        self.prompts.get(index)
    }

    pub fn get_secret(&self, index: usize) -> Option<&String> {
        self.secrets.get(index)
    }

    pub fn get_client_context(&self, index: usize) -> Option<&String> {
        self.client_context.get(index)
    }

    pub fn add_to_client_context(&mut self, key: &str) -> usize {
        match self.client_context.iter().position(|x| x == key) {
            Some(i) => i,
            None => {
                self.client_context.push(key.to_string());

                self.client_context.len() - 1
            }
        }
    }

    pub fn add_keys_to_client_context(&mut self, keys: Vec<String>) {
        self.client_context.extend(keys);
    }

    pub fn get_client_context_index(&self, name: &str) -> Option<(&String, u8)> {
        let index = self
            .client_context
            .iter()
            .position(|context_name| context_name == name);

        let result = index.map(|i| (self.client_context.get(i).unwrap(), i as u8));
        result
    }
}

/// The compiled bytecode for an expression
#[derive(Debug, Clone)]
pub struct ExprByteCode {
    codes: Vec<u8>,
    strings: Vec<String>,
}

impl ExprByteCode {
    pub fn new(codes: Vec<u8>, strings: Vec<String>) -> Self {
        Self { codes, strings }
    }

    pub fn codes(&self) -> &[u8] {
        &self.codes
    }

    pub fn strings(&self) -> &[String] {
        &self.strings
    }
}

/// Compile an [`ast::Expr`] into [`ExprByteCode`]
pub fn compile(expr: &(Expr, Range<usize>), env: &CompileTimeEnv) -> ExprResult<ExprByteCode> {
    let mut strings: Vec<String> = vec![];
    let codes = compile_expr(expr, env, &mut strings)?;
    Ok(ExprByteCode::new(codes, strings))
}

fn compile_expr(
    (expr, span): &(Expr, Range<usize>),
    env: &CompileTimeEnv,
    strings: &mut Vec<String>,
) -> ExprResult<Vec<u8>> {
    use opcode::*;

    let mut codes = vec![];
    let mut errs: Vec<(ExprError, Range<usize>)> = vec![];

    match expr {
        Expr::String(string) => {
            if let Some(index) = strings.iter().position(|x| x == &string.0) {
                codes.push(CONSTANT);
                codes.push(index as u8);
            } else {
                strings.push(string.0.clone());
                let index = strings.len() - 1;
                codes.push(CONSTANT);
                codes.push(index as u8);
            }
        }
        Expr::Identifier(identifier) => {
            let identifier_name = identifier.0.as_str();

            if let Some((_, index)) = env.get_builtin_index(identifier_name) {
                codes.push(GET);
                codes.push(lookup::BUILTIN);
                codes.push(index);
            } else if let Some((_, index)) = env.get_user_builtin_index(identifier_name) {
                codes.push(GET);
                codes.push(lookup::USER_BUILTIN);
                codes.push(index);
            } else {
                let identifier_prefix = &identifier_name[..1];
                let identifier_suffix = &identifier_name[1..];

                match identifier_prefix {
                    "?" => {
                        if let Some(index) = get(&env.prompts, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::PROMPT);
                            codes.push(index);
                        }
                    }
                    "!" => {
                        if let Some(index) = get(&env.secrets, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::SECRET);
                            codes.push(index);
                        }
                    }
                    ":" => {
                        if let Some(index) = get(&env.vars, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::VAR);
                            codes.push(index);
                        }
                    }
                    "@" => {
                        if let Some(index) = get(&env.client_context, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::CLIENT_CTX);
                            codes.push(index);
                        }
                    }
                    _ => {}
                };
            }
        }
        Expr::Call(expr_call) => {
            let callee_bytecode = compile_expr(&expr_call.callee, env, strings)?;

            if let Some(_op) = callee_bytecode.get(0) {
                if let Some(lookup) = callee_bytecode.get(1) {
                    if let Some(index) = callee_bytecode.get(2) {
                        match *lookup {
                            lookup::BUILTIN => {
                                let builtin = env.get_builtin((*index).into()).unwrap();

                                let call_arity: usize = expr_call.args.len();

                                if !builtin.arity_matches(call_arity.try_into().unwrap()) {
                                    errs.push((
                                        ExprError::TypeError(WrongNumberOfArgs {
                                            expected: builtin.arity().try_into().unwrap(),
                                            actual: call_arity,
                                        }),
                                        span.clone(),
                                    ));
                                }
                            }
                            lookup::USER_BUILTIN => {
                                let builtin = env.get_user_builtin((*index).into()).unwrap();

                                let call_arity: usize = expr_call.args.len();

                                if !builtin.arity_matches(call_arity.try_into().unwrap()) {
                                    errs.push((
                                        ExprError::TypeError(WrongNumberOfArgs {
                                            expected: builtin.arity().try_into().unwrap(),
                                            actual: call_arity,
                                        }),
                                        span.clone(),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            codes.extend(callee_bytecode);

            for arg in expr_call.args.iter() {
                let arg_bytecode = compile_expr(arg, env, strings)?;

                codes.extend(arg_bytecode);
            }

            codes.push(opcode::CALL);
            codes.push(expr_call.args.len() as u8);
        }
        Expr::Bool(value) => match value.0 {
            true => {
                codes.push(opcode::TRUE);
            }
            false => {
                codes.push(opcode::FALSE);
            }
        },
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    Ok(codes)
}

#[cfg(test)]
mod value_tests {
    use super::*;

    #[test]
    fn test_builtins_debug_0_arity() {
        assert_eq!(
            "builtin test_builtin(0)",
            format!(
                "{:#?}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    arity: FnArity::N(0),
                    func: Rc::new(|_| { Value::String("test_builtin".to_string()) })
                }
            )
        )
    }

    #[test]
    fn test_builtins_debug_1_arity() {
        assert_eq!(
            "builtin test_builtin(1)",
            format!(
                "{:#?}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    arity: FnArity::N(1),
                    func: Rc::new(|_| { Value::String("test_builtin".to_string()) })
                }
            )
        )
    }

    #[test]
    fn test_builtins_debug_2_arity() {
        assert_eq!(
            "builtin test_builtin(2)",
            format!(
                "{:#?}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    arity: FnArity::N(2),
                    func: Rc::new(|_| { Value::String("test_builtin".to_string()) })
                }
            )
        )
    }
}
