use std::rc::Rc;

use crate::{
    compiler::{self, BuiltinFn, Env, ExprByteCode},
    prelude::lookup,
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Fn(Rc<BuiltinFn>),
}

impl Value {
    pub fn get_string(&self) -> &str {
        match self {
            Value::String(s) => s.as_str(),
            _ => panic!("Value is not a string"),
        }
    }

    pub fn get_func(&self) -> Rc<BuiltinFn> {
        match self {
            Value::Fn(f) => f.clone(),
            _ => panic!("Value is not a function"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeEnv {
    pub vars: Vec<String>,
    pub prompts: Vec<String>,
    pub secrets: Vec<String>,
}

#[derive(Debug)]
pub struct Vm<'bytecode> {
    bytecode: Option<&'bytecode ExprByteCode>,
    ip: usize,
    stack: Vec<Value>,
}

impl<'bytecode> Vm<'bytecode> {
    pub fn new() -> Self {
        Self {
            bytecode: None,
            ip: 0,
            stack: vec![],
        }
    }

    pub fn interpret(
        &mut self,
        bytecode: &'bytecode ExprByteCode,
        env: &Env,
        runtime_env: &RuntimeEnv,
    ) -> Result<Value, ()> {
        self.bytecode = Some(bytecode);
        self.ip = 0;

        while let Some(op_code) = self
            .bytecode
            .and_then(|ExprByteCode { codes, strings: _ }| codes.get(self.ip))
        {
            self.interpret_op(env, runtime_env, *op_code);
        }

        assert_eq!(1, self.stack.len());

        let value = self.stack_pop();

        eprintln!("{value:#?}");

        Ok(value)
    }

    fn interpret_op(&mut self, env: &Env, runtime_env: &RuntimeEnv, op_code: u8) {
        match op_code {
            compiler::opcode::CALL => self.op_call(),
            compiler::opcode::CONSTANT => self.op_constant(),
            compiler::opcode::GET => self.op_get(env, &runtime_env),
            _ => panic!("Invalid OP code: {op_code}"),
        }
    }

    fn op_call(&mut self) {
        // Confirm the current op code is CALL
        assert_eq!(
            compiler::opcode::CALL,
            self.read_u8(),
            "Expected CALL opcode"
        );

        let arg_count = self.read_u8() as usize;

        let mut args: Vec<Value> = vec![];

        for _ in 0..arg_count {
            args.push(self.stack_pop());
        }

        args.reverse();

        let value = self.stack_pop();

        let builtin = value.get_func().func.clone();

        let result = builtin(args);

        self.stack_push(Value::String(result));
    }

    fn op_get(&mut self, env: &Env, runtime_env: &RuntimeEnv) {
        assert_eq!(compiler::opcode::GET, self.read_u8(), "Expected GET opcode");
        let get_lookup = self.read_u8();
        let get_idx = self.read_u8() as usize;

        match get_lookup {
            lookup::BUILTIN => {
                let value = env
                    .builtins
                    .get(get_idx)
                    .expect(&format! {"undefined builtin: {get_idx}"});
                self.stack_push(Value::Fn(value.clone()));
            }
            lookup::VAR => {
                let value = env
                    .vars
                    .get(get_idx)
                    .and_then(|_| runtime_env.vars.get(get_idx))
                    .expect(&format! {"undefined variable: {get_idx}"});

                self.stack_push(Value::String(value.clone()));
            }
            lookup::PROMPT => {
                let value = env
                    .prompts
                    .get(get_idx)
                    .and_then(|_| runtime_env.prompts.get(get_idx))
                    .expect(&format! {"undefined prompt: {get_idx}"});

                self.stack_push(Value::String(value.clone()));
            }
            lookup::SECRET => {
                let value = env
                    .secrets
                    .get(get_idx)
                    .and_then(|_| runtime_env.secrets.get(get_idx))
                    .expect(&format! {"undefined secret: {get_idx}"});

                self.stack_push(Value::String(value.clone()));
            }
            _ => panic!("invalid get lookup code: {}", get_lookup),
        };
    }

    fn op_constant(&mut self) {
        assert_eq!(
            compiler::opcode::CONSTANT,
            self.read_u8(),
            "Expected CONSTANT opcode"
        );

        let get_idx = self.read_u8() as usize;

        let s = self
            .bytecode
            .expect("should have bytecode")
            .strings
            .get(get_idx)
            .expect(&format!("undefined string: {}", get_idx));

        self.stack_push(Value::String(s.clone()));
    }

    fn stack_push(&mut self, value: Value) {
        eprintln!("Pushing value: {:?}", value);

        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> Value {
        let value = self
            .stack
            .pop()
            .expect("should have a value to pop from the stack");

        eprintln!("Popping value: {:?}", value);

        value
    }

    fn read_u8(&mut self) -> u8 {
        let current_ip = (self.ip as u8).clone();

        self.ip += 1;

        self.bytecode
            .expect("should have bytecode")
            .codes
            .get(current_ip as usize)
            .expect("should have op in bytecode at {}")
            .clone()
    }
}
