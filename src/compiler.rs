use core::fmt;
use std::rc::Rc;

use crate::{ast, vm::Value};

pub mod opcode {
    iota::iota! {
        pub const
        CALL: u8 = iota;,
        GET,
        CONSTANT
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
        f.debug_struct("BuiltinFn")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

impl<'a> From<&'a (String, u8)> for BuiltinFn {
    fn from(value: &'a (String, u8)) -> Self {
        BuiltinFn {
            name: value.0.clone(),
            arity: value.1.clone(),
            func: Rc::new(|_| "".into()),
        }
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
#[derive(Debug)]
pub struct ExprByteCode {
    pub codes: Vec<u8>,
    pub strings: Vec<String>,
}

impl ExprByteCode {
    pub fn new() -> Self {
        Self {
            codes: vec![],
            strings: vec![],
        }
    }

    pub fn from(codes: Vec<u8>) -> Self {
        Self {
            codes,
            strings: vec![],
        }
    }

    pub fn push_string(&mut self, string: &str) {
        self.strings.push(string.to_string());
    }
}

/// Compile an [`ast::Expr`] into [`ExprByteCode`]
pub fn compile(expr: &ast::Expr, env: &Env) -> ExprByteCode {
    let mut strings: Vec<String> = vec![];
    let codes = compile_expr(expr, env, &mut strings);
    ExprByteCode { codes, strings }
}

fn compile_expr(expr: &ast::Expr, env: &Env, strings: &mut Vec<String>) -> Vec<u8> {
    use opcode::*;

    let mut codes = vec![];

    match expr {
        ast::Expr::String(string) => {
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
        ast::Expr::Identifier(identifier) => {
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
        ast::Expr::Call(expr_call) => {
            codes.extend(compile_expr(&expr_call.callee.0, env, strings));

            for arg in expr_call.args.iter() {
                codes.extend(compile_expr(&arg.0, env, strings));
            }

            codes.push(opcode::CALL);
            codes.push(expr_call.args.len() as u8);
        }
    }

    codes
}
