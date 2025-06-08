use std::{fmt::Display, rc::Rc};

use crate::compiler::{
    BuiltinFn, Env, ExprByteCode,
    lookup::{BUILTIN, PROMPT, SECRET, VAR},
    opcode,
};

#[derive(Debug, Clone, PartialEq)]
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

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "`{}`", string),
            Value::Fn(builtin) => write!(f, "builtin {}({})", builtin.name, builtin.arity),
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
        env: &Env,
        runtime_env: &RuntimeEnv,
    ) -> Result<Value, ()> {
        self.bytecode = Some(bytecode.into());
        self.ip = 0;

        while let Some(op_code) = self.bytecode.as_ref().and_then(|bc| bc.codes.get(self.ip)) {
            self.interpret_op(env, runtime_env, *op_code);
        }

        assert_eq!(1, self.stack.len());

        let value = self.stack_pop();

        eprintln!("{value}");

        Ok(value)
    }

    fn interpret_op(&mut self, env: &Env, runtime_env: &RuntimeEnv, op_code: u8) {
        match op_code {
            opcode::CALL => self.op_call(),
            opcode::CONSTANT => self.op_constant(),
            opcode::GET => self.op_get(env, &runtime_env),
            _ => panic!("Invalid OP code: {op_code}"),
        }
    }

    fn op_call(&mut self) {
        // Confirm the current op code is CALL
        assert_eq!(opcode::CALL, self.read_u8(), "Expected CALL opcode");

        let arg_count = self.read_u8() as usize;

        let mut args: Vec<Value> = vec![];

        for _ in 0..arg_count {
            args.push(self.stack_pop());
        }

        args.reverse();

        let value = self.stack_pop();

        let builtin = value.get_func().func.clone();

        let result = builtin(args);

        self.stack_push(result);
    }

    fn op_get(&mut self, env: &Env, runtime_env: &RuntimeEnv) {
        assert_eq!(opcode::GET, self.read_u8(), "Expected GET opcode");
        let get_lookup = self.read_u8();
        let get_idx = self.read_u8() as usize;

        match get_lookup {
            BUILTIN => {
                let value = env
                    .get_builtin(get_idx)
                    .expect(&format! {"undefined builtin: {get_idx}"});
                self.stack_push(Value::Fn(value.clone()));
            }
            VAR => {
                let value = env
                    .get_var(get_idx)
                    .and_then(|_| runtime_env.vars.get(get_idx))
                    .expect(&format! {"undefined variable: {get_idx}"});

                self.stack_push(Value::String(value.clone()));
            }
            PROMPT => {
                let value = env
                    .get_prompt(get_idx)
                    .and_then(|_| runtime_env.prompts.get(get_idx))
                    .expect(&format! {"undefined prompt: {get_idx}"});

                self.stack_push(Value::String(value.clone()));
            }
            SECRET => {
                let value = env
                    .get_secret(get_idx)
                    .and_then(|_| runtime_env.secrets.get(get_idx))
                    .expect(&format! {"undefined secret: {get_idx}"});

                self.stack_push(Value::String(value.clone()));
            }
            _ => panic!("invalid get lookup code: {}", get_lookup),
        };
    }

    fn op_constant(&mut self) {
        assert_eq!(opcode::CONSTANT, self.read_u8(), "Expected CONSTANT opcode");

        let get_idx = self.read_u8() as usize;

        let s = self
            .bytecode
            .as_ref()
            .expect("should have bytecode")
            .strings
            .get(get_idx)
            .expect(&format!("undefined string: {}", get_idx));

        self.stack_push(Value::String(s.clone()));
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> Value {
        let value = self
            .stack
            .pop()
            .expect("should have a value to pop from the stack");

        value
    }

    fn read_u8(&mut self) -> u8 {
        let current_ip = (self.ip as u8).clone();

        self.ip += 1;

        self.bytecode
            .as_ref()
            .expect("should have bytecode")
            .codes
            .get(current_ip as usize)
            .expect("should have op in bytecode at {}")
            .clone()
    }
}
