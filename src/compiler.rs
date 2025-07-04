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
                        FnArg::new("a", Type::Value),
                        FnArg::new("b", Type::Value),
                        FnArg::new_varadic("rest", Type::Value),
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
                Rc::new(BuiltinFn {
                    name: String::from("eq"),
                    args: vec![FnArg::new("a", Type::Value), FnArg::new("b", Type::Value)],
                    return_type: Type::Bool,
                    func: Rc::new(BuiltinFns::eq),
                }),
                Rc::new(BuiltinFn {
                    name: String::from("not"),
                    args: vec![FnArg::new("value", Type::Bool)],
                    return_type: Type::Bool,
                    func: Rc::new(BuiltinFns::not),
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
    version: [u8; 4],
    codes: Vec<u8>,
    strings: Vec<String>,
}

impl ExprByteCode {
    pub fn new(codes: Vec<u8>, strings: Vec<String>) -> Self {
        let version_bytes = get_version_bytes();
        let version_bytes_from_codes = &codes[0..4];

        assert_eq!(
            version_bytes, version_bytes_from_codes,
            "Version bytes do not match"
        );

        let codes = codes[4..].to_vec();

        Self {
            version: version_bytes,
            codes,
            strings,
        }
    }

    pub fn version(&self) -> &[u8; 4] {
        &self.version
    }

    pub fn codes(&self) -> &[u8] {
        &self.codes
    }

    pub fn strings(&self) -> &[String] {
        &self.strings
    }
}

pub fn get_version_bytes() -> [u8; 4] {
    [
        env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        0,
    ]
}

/// Compile an [`ast::Expr`] into [`ExprByteCode`]
pub fn compile(expr: &ExprS, env: &CompileTimeEnv) -> ExprResult<ExprByteCode> {
    let mut strings: Vec<String> = vec![];
    let mut codes = vec![];

    codes.extend(get_version_bytes());

    codes.extend(compile_expr(expr, env, &mut strings)?);

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
                        } else {
                            errs.push((
                                CompileError::Undefined(identifier_name.to_string()).into(),
                                span.clone(),
                            ));
                        }
                    }
                    "!" => {
                        if let Some(index) = get(&env.secrets, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::SECRET);
                            codes.push(index);
                        } else {
                            errs.push((
                                CompileError::Undefined(identifier_name.to_string()).into(),
                                span.clone(),
                            ));
                        }
                    }
                    ":" => {
                        if let Some(index) = get(&env.vars, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::VAR);
                            codes.push(index);
                        } else {
                            errs.push((
                                CompileError::Undefined(identifier_name.to_string()).into(),
                                span.clone(),
                            ));
                        }
                    }
                    "@" => {
                        if let Some(index) = get(&env.client_context, identifier_suffix) {
                            codes.push(GET);
                            codes.push(lookup::CLIENT_CTX);
                            codes.push(index);
                        } else {
                            errs.push((
                                CompileError::Undefined(identifier_name.to_string()).into(),
                                span.clone(),
                            ));
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
            let callee_bytecode = compile_expr(&expr_call.callee, env, strings)?;

            if let Some(_op) = callee_bytecode.first()
                && let Some(lookup) = callee_bytecode.get(1)
                && let Some(index) = callee_bytecode.get(2)
            {
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

                        let args: Vec<_> = expr_call.args.iter().take(call_arity).collect();

                        for (i, fnarg) in builtin.args.iter().enumerate() {
                            if let Some((a, a_span)) = args.get(i) {
                                let a_type = a.get_type();

                                let types_match = fnarg.ty == a_type
                                    || fnarg.ty == Type::Value
                                    || a_type == Type::Unknown;

                                if !types_match {
                                    errs.push((
                                        CompileError::TypeMismatch {
                                            expected: fnarg.ty.clone(),
                                            actual: a_type.clone(),
                                        }
                                        .into(),
                                        a_span.clone(),
                                    ));
                                }
                            }
                        }
                    }
                    lookup::USER_BUILTIN => {
                        let builtin = env.get_user_builtin((*index).into()).unwrap();

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
                    _ => {
                        errs.push((
                            CompileError::InvalidLookupType(*lookup).into(),
                            span.clone(),
                        ));
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

#[cfg(test)]
mod compiler_tests {
    use super::*;

    #[test]
    pub fn current_version_bytes() {
        let version_bytes = get_version_bytes();

        assert_eq!(version_bytes, [0, 7, 0, 0]);
    }

    #[test]
    pub fn valid_bytecode_version_bytes() {
        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::TRUE);

        ExprByteCode::new(codes.to_vec(), vec![]);
    }

    #[test]
    #[should_panic(expected = "Version bytes do not match")]
    pub fn invalid_bytecode_version_bytes() {
        let mut codes: Vec<u8> = [0, 0, 0, 0].to_vec();
        codes.push(opcode::TRUE);

        ExprByteCode::new(codes.to_vec(), vec![]);
    }

    #[test]
    pub fn get_version_bytes_from_bytecode() {
        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::TRUE);

        let bytecode = ExprByteCode::new(codes.to_vec(), vec![]);

        assert_eq!(bytecode.version(), &get_version_bytes());
    }
}
