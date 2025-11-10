//! The virtual machine and associated types

use crate::{
    compiler::{
        CompileTimeEnv, ExprByteCode,
        lookup::{BUILTIN, PROMPT, SECRET, TYPE, VAR},
        opcode,
    },
    errors::{ExprErrorS, ExprResult, RuntimeError},
    prelude::lookup::{CLIENT_CTX, USER_BUILTIN},
    types::Type,
    value::Value,
};

#[derive(Debug, Clone, Default)]
pub struct RuntimeEnv {
    pub vars: Vec<String>,
    pub prompts: Vec<String>,
    pub secrets: Vec<String>,
    pub client_context: Vec<Value>,
}

impl RuntimeEnv {
    pub fn add_to_client_context(&mut self, index: usize, value: Value) {
        if index < self.client_context.len() {
            self.client_context[index] = value;
        } else {
            self.client_context.push(value);
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    bytecode: Option<Box<ExprByteCode>>,
    ip: usize,
    stack: Vec<Value>,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        Self {
            bytecode: None,
            ip: 0,
            stack: vec![],
        }
    }

    pub fn interpret(
        &mut self,
        bytecode: Box<ExprByteCode>,
        env: &CompileTimeEnv,
        runtime_env: &RuntimeEnv,
    ) -> ExprResult<Value> {
        self.bytecode = Some(bytecode);
        self.ip = 0;

        let mut errs: Vec<ExprErrorS> = vec![];

        while let Some(op_code) = self
            .bytecode
            .as_ref()
            .and_then(|bc| bc.codes().get(self.ip))
        {
            if let Err(e) = self.interpret_op(env, runtime_env, *op_code) {
                errs.extend(e);
            }
        }

        if !errs.is_empty() {
            return Err(errs);
        }

        self.stack_pop()
    }

    fn interpret_op(
        &mut self,
        env: &CompileTimeEnv,
        runtime_env: &RuntimeEnv,
        op_code: u8,
    ) -> ExprResult<()> {
        match op_code {
            opcode::CALL => self.op_call(),
            opcode::CONSTANT => self.op_constant(),
            opcode::GET => self.op_get(env, runtime_env),
            opcode::TRUE => self.op_true(),
            opcode::FALSE => self.op_false(),
            _ => panic!("Invalid OP code: {op_code}"),
        }
    }

    fn op_call(&mut self) -> ExprResult<()> {
        // Consume current op: CALL
        self.read_u8();

        let arg_count = self.read_u8() as usize;

        let mut args: Vec<Value> = vec![];

        for _ in 0..arg_count {
            args.push(self.stack_pop()?);
        }

        args.reverse();

        let value = self.stack_pop()?;

        let builtin = value.get_func()?.func;

        let result = builtin(args);

        self.stack_push(result?);

        Ok(())
    }

    fn op_get(&mut self, env: &CompileTimeEnv, runtime_env: &RuntimeEnv) -> ExprResult<()> {
        // Consume current op: GET
        self.read_u8();

        let get_lookup = self.read_u8();
        let get_idx = self.read_u8() as usize;

        match get_lookup {
            BUILTIN => {
                let value = env
                    .get_builtin(get_idx)
                    .unwrap_or_else(|| panic!("undefined builtin: {get_idx}"));
                self.stack_push(Value::Fn(value.clone().into()));
            }
            USER_BUILTIN => {
                let value = env
                    .get_user_builtin(get_idx)
                    .unwrap_or_else(|| panic!("undefined user builtin: {get_idx}"));
                self.stack_push(Value::Fn(value.clone().into()));
            }
            VAR => {
                let value = env
                    .get_var(get_idx)
                    .and_then(|_| runtime_env.vars.get(get_idx))
                    .unwrap_or_else(|| panic!("undefined variable: {get_idx}"));

                self.stack_push(Value::String(value.clone()));
            }
            PROMPT => {
                let value = env
                    .get_prompt(get_idx)
                    .and_then(|_| runtime_env.prompts.get(get_idx))
                    .unwrap_or_else(|| panic!("undefined prompt: {get_idx}"));

                self.stack_push(Value::String(value.clone()));
            }
            SECRET => {
                let value = env
                    .get_secret(get_idx)
                    .and_then(|_| runtime_env.secrets.get(get_idx))
                    .unwrap_or_else(|| panic!("undefined secret: {get_idx}"));

                self.stack_push(Value::String(value.clone()));
            }
            CLIENT_CTX => {
                let value = env
                    .get_client_context(get_idx)
                    .and_then(|_| runtime_env.client_context.get(get_idx))
                    .unwrap_or_else(|| panic!("undefined client context: {get_idx}"));

                self.stack_push(value.clone());
            }
            TYPE => {
                let ty = self
                    .bytecode
                    .as_ref()
                    .unwrap()
                    .types()
                    .get(get_idx)
                    .unwrap_or_else(|| panic!("undefined type: {get_idx}"));

                if ty.is_type() {
                    self.stack_push(Value::Type(ty.clone().into()));
                } else {
                    self.stack_push(Value::Type(Type::Type(ty.clone().into()).into()));
                }
            }
            _ => panic!("Invalid get lookup code: {get_lookup}"),
        };

        Ok(())
    }

    fn op_constant(&mut self) -> ExprResult<()> {
        // Consume current op: CONSTANT
        self.read_u8();

        let get_idx = self.read_u8() as usize;

        let s = self
            .bytecode
            .as_ref()
            .expect("should have bytecode")
            .constants()
            .get(get_idx)
            .unwrap_or_else(|| panic!("undefined constant: {get_idx}"));

        self.stack_push(s.clone());

        Ok(())
    }

    fn op_true(&mut self) -> ExprResult<()> {
        // Consume current op: TRUE
        self.read_u8();

        self.stack_push(Value::Bool(true));

        Ok(())
    }

    fn op_false(&mut self) -> ExprResult<()> {
        // Consume current op: FALSE
        self.read_u8();

        self.stack_push(Value::Bool(false));

        Ok(())
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> ExprResult<Value> {
        if let Some(value) = self.stack.pop() {
            return Ok(value);
        };

        Err(vec![(RuntimeError::EmptyStack.into(), 0..0)])
    }

    fn read_u8(&mut self) -> u8 {
        let current_ip = self.ip as u8;

        self.ip += 1;

        *self
            .bytecode
            .as_ref()
            .expect("should have bytecode")
            .codes()
            .get(current_ip as usize)
            .expect("should have op in bytecode at {}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{compiler::get_version_bytes, errors::ExprError, prelude::lookup};

    use super::*;

    #[test]
    fn test_popping_from_empty_stack() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();

        // Get builtin function `id`
        codes.push(opcode::GET);
        codes.push(lookup::BUILTIN);
        codes.push(0);

        // Specify call will be passing 1 argument
        // but don't push the bytecode for the argument passed
        codes.push(opcode::CALL);
        codes.push(1);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        assert_eq!(
            Err(vec![(
                ExprError::RuntimeError(RuntimeError::EmptyStack),
                0..0
            )]),
            vm.interpret(bytecode, &env, &runtime_env)
        );
    }

    #[test]
    #[should_panic(expected = "Invalid OP code: 99")]
    fn test_invalid_opcode_99() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(99);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        // Attempt to interpret the bytecode, expecting a panic due to invalid opcode 99
        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "Invalid get lookup code: 99")]
    fn test_invalid_look_99() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(99);
        codes.push(0);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        // Attempt to interpret the bytecode, expecting a panic due to invalid opcode 99
        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined variable: 99")]
    fn undefined_variable() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::VAR);
        codes.push(99);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined prompt: 99")]
    fn undefined_prompt() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::PROMPT);
        codes.push(99);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined secret: 99")]
    fn undefined_secret() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::SECRET);
        codes.push(99);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined builtin: 255")]
    fn undefined_builtin() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::BUILTIN);
        codes.push(255);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined user builtin: 255")]
    fn undefined_user_builtin() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::USER_BUILTIN);
        codes.push(255);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined client context: 255")]
    fn undefined_client_context() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::CLIENT_CTX);
        codes.push(255);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined type: 255")]
    fn undefined_type() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::GET);
        codes.push(lookup::TYPE);
        codes.push(255);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "undefined constant: 255")]
    fn undefined_constant() {
        let mut vm = Vm::new();

        let mut codes = get_version_bytes().to_vec();
        codes.push(opcode::CONSTANT);
        codes.push(255);

        let bytecode = Box::new(ExprByteCode::new(codes, vec![], vec![]));
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }
}
