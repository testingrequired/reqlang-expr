//! The compiler and associated types

use core::fmt;
use std::rc::Rc;

use crate::{ast::Expr, errors::ExprResult, vm::Value};

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
        SECRET
    }
}

/// Try to get a string from a list
fn get(list: &Vec<String>, identifier: &str) -> Option<u8> {
    list.into_iter()
        .position(|x| x == identifier)
        .map(|i| i as u8)
}

/// Builtin function used in expressions
pub struct BuiltinFn {
    // Needs to follow identifier naming rules
    pub name: String,
    // Number of arguments the function expects
    pub arity: u8,
    // Function used at runtime
    pub func: Rc<dyn Fn(Vec<Value>) -> Value>,
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
}

#[derive(Debug)]
pub struct Env {
    builtins: Vec<Rc<BuiltinFn>>,
    vars: Vec<String>,
    prompts: Vec<String>,
    secrets: Vec<String>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            builtins: vec![
                Rc::new(BuiltinFn {
                    name: String::from("id"),
                    arity: 1,
                    func: Rc::new(BuiltinFns::id),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("noop"),
                    arity: 0,
                    func: Rc::new(BuiltinFns::noop),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("is_empty"),
                    arity: 1,
                    func: Rc::new(BuiltinFns::is_empty),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("not"),
                    arity: 1,
                    func: Rc::new(BuiltinFns::not),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("and"),
                    arity: 2,
                    func: Rc::new(BuiltinFns::and),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("or"),
                    arity: 2,
                    func: Rc::new(BuiltinFns::or),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("cond"),
                    arity: 3,
                    func: Rc::new(BuiltinFns::cond),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("to_str"),
                    arity: 1,
                    func: Rc::new(BuiltinFns::to_str),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("concat"),
                    arity: 10,
                    func: Rc::new(BuiltinFns::concat),
                }),
            ],
            vars: Vec::new(),
            prompts: Vec::new(),
            secrets: Vec::new(),
        }
    }
}

impl Env {
    pub fn new(vars: Vec<String>, prompts: Vec<String>, secrets: Vec<String>) -> Self {
        let mut env = Self::default();

        env.vars = vars;
        env.prompts = prompts;
        env.secrets = secrets;

        env
    }

    pub fn get_builtin_index(&self, name: &str) -> Option<(&Rc<BuiltinFn>, u8)> {
        let index = self.builtins.iter().position(|x| x.name == name);

        let result = index.map(|i| (self.builtins.get(i).unwrap(), i as u8));
        result
    }

    pub fn add_builtins(&mut self, builtins: Vec<Rc<BuiltinFn>>) {
        for builtin in builtins {
            self.add_builtin(builtin);
        }
    }

    pub fn add_builtin(&mut self, builtin: Rc<BuiltinFn>) {
        self.builtins.push(builtin);
    }

    pub fn get_builtin(&self, index: usize) -> Option<&Rc<BuiltinFn>> {
        self.builtins.get(index)
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
pub fn compile(expr: &Expr, env: &Env) -> ExprResult<ExprByteCode> {
    let mut strings: Vec<String> = vec![];
    let codes = compile_expr(expr, env, &mut strings)?;
    Ok(ExprByteCode::new(codes, strings))
}

fn compile_expr(expr: &Expr, env: &Env, strings: &mut Vec<String>) -> ExprResult<Vec<u8>> {
    use opcode::*;

    let mut codes = vec![];

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
                    _ => {}
                };
            }
        }
        Expr::Call(expr_call) => {
            let callee_bytecode = compile_expr(&expr_call.callee.0, env, strings)?;

            codes.extend(callee_bytecode);

            for arg in expr_call.args.iter() {
                let arg_bytecode = compile_expr(&arg.0, env, strings)?;

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
                    arity: 0,
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
                    arity: 1,
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
                    arity: 2,
                    func: Rc::new(|_| { Value::String("test_builtin".to_string()) })
                }
            )
        )
    }
}
