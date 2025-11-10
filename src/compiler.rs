//! The compiler and associated types

use crate::{
    ast::{Expr, ExprS, IdentifierKind, add_type_to_expr},
    builtins::BuiltinFn,
    errors::{
        CompileError::{self, WrongNumberOfArgs},
        ExprError, ExprErrorS, ExprResult,
    },
    prelude::lookup::TYPE,
    types::Type,
    value::Value,
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
        CLIENT_CTX,
        TYPE
    }
}

/// Try to get a string from a list
fn get(list: &[String], identifier: &str) -> Option<u8> {
    list.iter().position(|x| x == identifier).map(|i| i as u8)
}

#[derive(Debug)]
pub struct CompileTimeEnv {
    builtins: Vec<BuiltinFn<'static>>,
    user_builtins: Vec<BuiltinFn<'static>>,
    vars: Vec<String>,
    prompts: Vec<String>,
    secrets: Vec<String>,
    client_context: Vec<String>,
}

impl Default for CompileTimeEnv {
    fn default() -> Self {
        Self {
            builtins: BuiltinFn::DEFAULT_BUILTINS.to_vec(),
            user_builtins: vec![],
            vars: vec![],
            prompts: vec![],
            secrets: vec![],
            client_context: vec![],
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

    pub fn get_builtin_index(&self, name: &str) -> Option<(&BuiltinFn, u8)> {
        let index = self.builtins.iter().position(|x| x.name == name);

        let result = index.map(|i| (self.builtins.get(i).unwrap(), i as u8));
        result
    }

    pub fn get_user_builtin_index(&self, name: &str) -> Option<(&BuiltinFn, u8)> {
        let index = self.user_builtins.iter().position(|x| x.name == name);

        let result = index.map(|i| (self.user_builtins.get(i).unwrap(), i as u8));
        result
    }

    pub fn add_user_builtins(&mut self, builtins: Vec<BuiltinFn<'static>>) {
        for builtin in builtins {
            self.add_user_builtin(builtin);
        }
    }

    pub fn add_user_builtin(&mut self, builtin: BuiltinFn<'static>) {
        self.user_builtins.push(builtin);
    }

    pub fn get_builtin(&self, index: usize) -> Option<&BuiltinFn<'static>> {
        self.builtins.get(index)
    }

    pub fn get_user_builtin(&self, index: usize) -> Option<&BuiltinFn<'static>> {
        self.user_builtins.get(index)
    }

    pub fn get_var(&self, index: usize) -> Option<&String> {
        self.vars.get(index)
    }

    pub fn get_var_index(&self, name: &str) -> Option<usize> {
        let index = self
            .vars
            .iter()
            .position(|context_name| context_name == name);

        index
    }

    pub fn get_prompt(&self, index: usize) -> Option<&String> {
        self.prompts.get(index)
    }

    pub fn get_prompt_index(&self, name: &str) -> Option<usize> {
        let index = self
            .prompts
            .iter()
            .position(|context_name| context_name == name);

        index
    }

    pub fn get_secret(&self, index: usize) -> Option<&String> {
        self.secrets.get(index)
    }

    pub fn get_secret_index(&self, name: &str) -> Option<usize> {
        let index = self
            .secrets
            .iter()
            .position(|context_name| context_name == name);

        index
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
    constants: Vec<Value>,
    types: Vec<Type>,
}

impl ExprByteCode {
    pub fn new(codes: Vec<u8>, constants: Vec<Value>, types: Vec<Type>) -> Self {
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
            constants,
            types,
        }
    }

    pub fn version(&self) -> &[u8; 4] {
        &self.version
    }

    pub fn codes(&self) -> &[u8] {
        &self.codes
    }

    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

    pub fn types(&self) -> &[Type] {
        &self.types
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
pub fn compile(expr: &mut ExprS, env: &CompileTimeEnv) -> ExprResult<ExprByteCode> {
    let mut constants: Vec<Value> = vec![];
    let mut types: Vec<Type> = vec![];
    let mut codes = vec![];

    codes.extend(get_version_bytes());

    codes.extend(compile_expr(expr, env, &mut constants, &mut types)?);

    Ok(ExprByteCode::new(codes, constants, types))
}

fn compile_expr(
    (expr, span): &mut ExprS,
    env: &CompileTimeEnv,
    constants: &mut Vec<Value>,
    types: &mut Vec<Type>,
) -> ExprResult<Vec<u8>> {
    use opcode::*;

    let mut codes = vec![];
    let mut errs: Vec<ExprErrorS> = vec![];

    add_type_to_expr(expr, env);

    match expr {
        Expr::String(string) => {
            if let Some(index) = constants.iter().position(|x| {
                if let Value::String(string_constant) = x {
                    string_constant == &string.0
                } else {
                    false
                }
            }) {
                codes.push(CONSTANT);
                codes.push(index as u8);
            } else {
                constants.push(Value::String(string.0.clone()));
                let index = constants.len() - 1;
                codes.push(CONSTANT);
                codes.push(index as u8);
            }
        }
        Expr::Number(number) => {
            if let Some(index) = constants.iter().position(|x| {
                if let Value::Number(value) = x {
                    value == &number.0
                } else {
                    false
                }
            }) {
                codes.push(CONSTANT);
                codes.push(index as u8);
            } else {
                constants.push(Value::Number(number.0.clone()));
                let index = constants.len() - 1;
                codes.push(CONSTANT);
                codes.push(index as u8);
            }
        }
        Expr::Identifier(identifier) => {
            let identifier_lookup_name = identifier.lookup_name();
            let identifier_name = identifier.full_name().to_string();

            let identifier_undefined_err = (
                CompileError::Undefined(identifier_name.clone()).into(),
                span.clone(),
            );

            let result = match identifier.identifier_kind() {
                IdentifierKind::Var => get(&env.vars, identifier_lookup_name).map(|index| {
                    codes.push(GET);
                    codes.push(lookup::VAR);
                    codes.push(index);
                }),
                IdentifierKind::Prompt => get(&env.prompts, identifier_lookup_name).map(|index| {
                    codes.push(GET);
                    codes.push(lookup::PROMPT);
                    codes.push(index);
                }),
                IdentifierKind::Secret => get(&env.secrets, identifier_lookup_name).map(|index| {
                    codes.push(GET);
                    codes.push(lookup::SECRET);
                    codes.push(index);
                }),
                IdentifierKind::Client => {
                    get(&env.client_context, identifier_lookup_name).map(|index| {
                        codes.push(GET);
                        codes.push(lookup::CLIENT_CTX);
                        codes.push(index);
                    })
                }
                IdentifierKind::Builtin => {
                    if let Some((_, index)) = env.get_builtin_index(identifier_lookup_name) {
                        codes.push(GET);
                        codes.push(lookup::BUILTIN);
                        codes.push(index);

                        Some(())
                    } else if let Some((_, index)) =
                        env.get_user_builtin_index(identifier_lookup_name)
                    {
                        codes.push(GET);
                        codes.push(lookup::USER_BUILTIN);
                        codes.push(index);

                        Some(())
                    } else {
                        None
                    }
                }
                IdentifierKind::Type => {
                    let ty = Type::from(&identifier_name);
                    if let Some(index) = types.iter().position(|x| x == &ty) {
                        codes.push(GET);
                        codes.push(TYPE);
                        codes.push(index as u8);
                    } else {
                        types.push(ty);
                        let index = types.len() - 1;
                        codes.push(GET);
                        codes.push(TYPE);
                        codes.push(index as u8);
                    }

                    Some(())
                }
            };

            if let None = result {
                errs.push(identifier_undefined_err);
            }
        }
        Expr::Call(expr_call) => {
            let callee_bytecode = compile_expr(&mut expr_call.callee, env, constants, types)?;

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
                    lookup::CLIENT_CTX => {
                        // No validation needs to be ran at this point
                        // This won't happen until runtime when the client
                        // a value.
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

            for arg in expr_call.args.iter_mut() {
                match compile_expr(arg, env, constants, types) {
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

        assert_eq!(version_bytes, [0, 8, 0, 0]);
    }

    #[test]
    pub fn valid_bytecode_version_bytes() {
        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::TRUE);

        ExprByteCode::new(codes.to_vec(), vec![], vec![]);
    }

    #[test]
    #[should_panic(expected = "Version bytes do not match")]
    pub fn invalid_bytecode_version_bytes() {
        let mut codes: Vec<u8> = [0, 0, 0, 0].to_vec();
        codes.push(opcode::TRUE);

        ExprByteCode::new(codes.to_vec(), vec![], vec![]);
    }

    #[test]
    pub fn get_version_bytes_from_bytecode() {
        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::TRUE);

        let bytecode = ExprByteCode::new(codes.to_vec(), vec![], vec![]);

        assert_eq!(bytecode.version(), &get_version_bytes());
    }
}
