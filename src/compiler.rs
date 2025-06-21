//! The compiler and associated types

use std::rc::Rc;

use crate::{
    ast::{Expr, ExprS},
    builtins::{BuiltinFn, BuiltinFns},
    errors::{
        CompileError::{self, WrongNumberOfArgs},
        ExprError, ExprErrorS, ExprResult,
    },
    prelude::FnArg,
    types::Type,
};

pub mod opcode {
    iota::iota! {
        pub const
        CALL: u8 = iota;,
        GET,
        CONSTANT,
        TRUE,
        FALSE,
        NOT,
        EQ
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
                    args: vec![FnArg::new("value", Type::Value)],
                    return_type: Type::Value,
                    func: Rc::new(BuiltinFns::id),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("noop"),
                    args: vec![],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::noop),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("is_empty"),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::is_empty),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("and"),
                    args: vec![FnArg::new("a", Type::Bool), FnArg::new("b", Type::Bool)],
                    return_type: Type::Bool,
                    func: Rc::new(BuiltinFns::and),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("or"),
                    args: vec![FnArg::new("a", Type::Bool), FnArg::new("b", Type::Bool)],
                    return_type: Type::Bool,
                    func: Rc::new(BuiltinFns::or),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("cond"),
                    args: vec![
                        FnArg::new("cond", Type::Bool),
                        FnArg::new("then", Type::Value),
                        FnArg::new("else", Type::Value),
                    ],
                    return_type: Type::Bool,
                    func: Rc::new(BuiltinFns::cond),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("to_str"),
                    args: vec![FnArg::new("value", Type::Value)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::to_str),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("concat"),
                    args: vec![
                        FnArg::new("a", Type::String),
                        FnArg::new("b", Type::String),
                        FnArg::new_varadic("rest", Type::String),
                    ],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::concat),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("contains"),
                    args: vec![
                        FnArg::new("needle", Type::String),
                        FnArg::new("haystack", Type::String),
                    ],
                    return_type: Type::Bool,
                    func: Rc::new(BuiltinFns::contains),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("trim"),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::trim),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("trim_start"),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::trim_start),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("trim_end"),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::trim_end),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("lowercase"),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::lowercase),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("uppercase"),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::uppercase),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("type"),
                    args: vec![FnArg::new("value", Type::Value)],
                    return_type: Type::String,
                    func: Rc::new(BuiltinFns::get_type),
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
#[derive(Debug, Clone, PartialEq)]
pub struct ExprByteCode {
    pub codes: Vec<u8>,
    pub strings: Vec<String>,
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
pub fn compile(expr: &ExprS, env: &CompileTimeEnv) -> ExprResult<ExprByteCode> {
    let mut strings: Vec<String> = vec![];
    let codes = compile_expr(expr, env, &mut strings)?;
    Ok(ExprByteCode::new(codes, strings))
}

fn compile_expr(
    (expr, span): &ExprS,
    env: &CompileTimeEnv,
    strings: &mut Vec<String>,
) -> ExprResult<Vec<u8>> {
    use opcode::*;

    let mut codes = vec![];
    let mut errs: Vec<ExprErrorS> = vec![];

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
                    _ => {
                        errs.push((
                            ExprError::CompileError(CompileError::Undefined(
                                identifier_name.to_string(),
                            )),
                            span.clone(),
                        ));
                    }
                };
            }
        }
        Expr::Call(expr_call) => {
            let identifier_name = expr_call.callee.0.identifier_name().unwrap_or_default();

            match identifier_name {
                "eq" => {
                    if expr_call.args.is_empty() {
                        errs.push((
                            ExprError::CompileError(WrongNumberOfArgs {
                                expected: 2,
                                actual: 0,
                            }),
                            span.clone(),
                        ));
                    } else if expr_call.args.len() > 2 {
                        errs.push((
                            ExprError::CompileError(WrongNumberOfArgs {
                                expected: 2,
                                actual: expr_call.args.len(),
                            }),
                            span.clone(),
                        ));
                    } else {
                        let arg = expr_call.args.first().expect("should have first argument");

                        match compile_expr(arg, env, strings) {
                            Ok(arg_bytecode) => {
                                codes.extend(arg_bytecode);
                            }
                            Err(err) => {
                                errs.extend(err);
                            }
                        }

                        let arg2 = expr_call.args.get(1).expect("should have second argument");

                        match compile_expr(arg2, env, strings) {
                            Ok(arg_bytecode) => {
                                codes.extend(arg_bytecode);
                            }
                            Err(err) => {
                                errs.extend(err);
                            }
                        }

                        codes.push(opcode::EQ);
                    }
                }
                "not" => {
                    if expr_call.args.is_empty() {
                        errs.push((
                            ExprError::CompileError(WrongNumberOfArgs {
                                expected: 1,
                                actual: 0,
                            }),
                            span.clone(),
                        ));
                    } else if expr_call.args.len() > 1 {
                        errs.push((
                            ExprError::CompileError(WrongNumberOfArgs {
                                expected: 1,
                                actual: expr_call.args.len(),
                            }),
                            span.clone(),
                        ));

                        let arg = expr_call.args.first().expect("should have first argument");

                        if !arg.0.is_bool() {
                            errs.push((
                                CompileError::TypeMismatch {
                                    expected: Type::Bool,
                                    actual: arg.0.get_type(),
                                }
                                .into(),
                                arg.1.clone(),
                            ));
                        }
                    } else {
                        let arg = expr_call.args.first().expect("should have first argument");
                        if !arg.0.is_bool() {
                            errs.push((
                                CompileError::TypeMismatch {
                                    expected: Type::Bool,
                                    actual: arg.0.get_type(),
                                }
                                .into(),
                                arg.1.clone(),
                            ));
                        }

                        match compile_expr(arg, env, strings) {
                            Ok(arg_bytecode) => {
                                codes.extend(arg_bytecode);
                            }
                            Err(err) => {
                                errs.extend(err);
                            }
                        }

                        codes.push(opcode::NOT);
                    }
                }
                _ => {
                    let callee_bytecode = compile_expr(&expr_call.callee, env, strings)?;

                    if let Some(_op) = callee_bytecode.first() {
                        if let Some(lookup) = callee_bytecode.get(1) {
                            if let Some(index) = callee_bytecode.get(2) {
                                match *lookup {
                                    lookup::BUILTIN => {
                                        let builtin = env.get_builtin((*index).into()).unwrap();

                                        let call_arity: usize = expr_call.args.len();

                                        if !builtin.arity_matches(call_arity.try_into().unwrap()) {
                                            errs.push((
                                                ExprError::CompileError(WrongNumberOfArgs {
                                                    expected: builtin.arity() as usize,
                                                    actual: call_arity,
                                                }),
                                                span.clone(),
                                            ));
                                        }
                                    }
                                    lookup::USER_BUILTIN => {
                                        let builtin =
                                            env.get_user_builtin((*index).into()).unwrap();

                                        let call_arity: usize = expr_call.args.len();

                                        if !builtin.arity_matches(call_arity.try_into().unwrap()) {
                                            errs.push((
                                                ExprError::CompileError(WrongNumberOfArgs {
                                                    expected: builtin.arity() as usize,
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
                        match compile_expr(arg, env, strings) {
                            Ok(arg_bytecode) => {
                                codes.extend(arg_bytecode);
                            }
                            Err(err) => {
                                errs.extend(err);
                            }
                        }
                    }

                    codes.push(opcode::CALL);
                    codes.push(expr_call.args.len() as u8);
                }
            }
        }
        Expr::Bool(value) => match value.0 {
            true => {
                codes.push(opcode::TRUE);
            }
            false => {
                codes.push(opcode::FALSE);
            }
        },
        Expr::Error => panic!("tried to compile despite parser errors"),
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    Ok(codes)
}
