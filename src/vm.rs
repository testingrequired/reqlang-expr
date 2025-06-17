//! The virtual machine and associated types

use std::ops::Range;

use crate::{
    compiler::{
        CompileTimeEnv, ExprByteCode,
        lookup::{BUILTIN, PROMPT, SECRET, VAR},
        opcode,
    },
    errors::{ExprError, ExprResult},
    prelude::lookup::{CLIENT_CTX, USER_BUILTIN},
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

        let mut errs: Vec<(ExprError, Range<usize>)> = vec![];

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
            return Err(errs.into());
        }

        let value = self.stack_pop()?;

        Ok(value)
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
        // Confirm the current op code is CALL
        assert_eq!(opcode::CALL, self.read_u8(), "Expected CALL opcode");

        let arg_count = self.read_u8() as usize;

        let mut args: Vec<Value> = vec![];

        for _ in 0..arg_count {
            args.push(self.stack_pop()?);
        }

        args.reverse();

        let value = self.stack_pop()?;

        let builtin = value.get_func().func.clone();

        let result = builtin(args);

        self.stack_push(result);

        Ok(())
    }

    fn op_get(&mut self, env: &CompileTimeEnv, runtime_env: &RuntimeEnv) -> ExprResult<()> {
        assert_eq!(opcode::GET, self.read_u8(), "Expected GET opcode");
        let get_lookup = self.read_u8();
        let get_idx = self.read_u8() as usize;

        match get_lookup {
            BUILTIN => {
                let value = env
                    .get_builtin(get_idx)
                    .unwrap_or_else(|| panic!("undefined builtin: {get_idx}"));
                self.stack_push(Value::Fn(value.clone()));
            }
            USER_BUILTIN => {
                let value = env
                    .get_user_builtin(get_idx)
                    .unwrap_or_else(|| panic!("undefined user builtin: {get_idx}"));
                self.stack_push(Value::Fn(value.clone()));
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
            _ => panic!("Invalid get lookup code: {}", get_lookup),
        };

        Ok(())
    }

    fn op_constant(&mut self) -> ExprResult<()> {
        assert_eq!(opcode::CONSTANT, self.read_u8(), "Expected CONSTANT opcode");

        let get_idx = self.read_u8() as usize;

        let s = self
            .bytecode
            .as_ref()
            .expect("should have bytecode")
            .strings()
            .get(get_idx)
            .unwrap_or_else(|| panic!("undefined string: {}", get_idx));

        self.stack_push(Value::String(s.clone()));

        Ok(())
    }

    fn op_true(&mut self) -> ExprResult<()> {
        assert_eq!(opcode::TRUE, self.read_u8(), "Expected TRUE opcode");

        self.stack_push(Value::Bool(true));

        Ok(())
    }

    fn op_false(&mut self) -> ExprResult<()> {
        assert_eq!(opcode::FALSE, self.read_u8(), "Expected FALSE opcode");

        self.stack_push(Value::Bool(false));

        Ok(())
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> ExprResult<Value> {
        Ok(self
            .stack
            .pop()
            .expect("should have a value to pop from the stack"))
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
    use super::*;

    #[test]
    #[should_panic(expected = "Invalid OP code: 99")]
    fn test_invalid_opcode_99() {
        let mut vm = Vm::new();

        let bytecode = Box::new(ExprByteCode::new(vec![99], vec![])); // 99 as invalid opcode
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        // Attempt to interpret the bytecode, expecting a panic due to invalid opcode 99
        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }

    #[test]
    #[should_panic(expected = "Invalid get lookup code: 99")]
    fn test_invalid_look_99() {
        let mut vm = Vm::new();

        let bytecode = Box::new(ExprByteCode::new(vec![opcode::GET, 99, 0], vec![])); // 99 as invalid opcode
        let env = CompileTimeEnv::default();
        let runtime_env = RuntimeEnv::default();

        // Attempt to interpret the bytecode, expecting a panic due to invalid opcode 99
        let _ = vm.interpret(bytecode, &env, &runtime_env);
    }
}
