use crate::compiler::{self, Env, ExprByteCode, Fn};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StackValue {
    String(String),
    Fn(Box<Fn>),
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
    stack: Vec<StackValue>,
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
    ) -> Result<StackValue, ()> {
        self.bytecode = Some(bytecode);
        self.ip = 0;

        while let Some(op_code) = self
            .bytecode
            .and_then(|ExprByteCode { codes }| codes.get(self.ip))
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
            compiler::opcode::CALL => self.op_call(env, runtime_env),
            compiler::opcode::BUILTIN => self.op_builtin(env),
            compiler::opcode::VAR => self.op_var(env, &runtime_env.vars),
            compiler::opcode::PROMPT => self.op_prompt(env),
            compiler::opcode::SECRET => self.op_secret(env),
            _ => panic!("Invalid OP code: {op_code}"),
        }
    }

    fn op_call(&mut self, env: &Env, runtime_env: &RuntimeEnv) {
        // Confirm the current op code is CALL
        assert_eq!(
            compiler::opcode::CALL,
            self.read_u8(),
            "Expected CALL opcode"
        );

        self.op_builtin(env);

        let arg_count = self.read_u8() as usize;

        // Push a placeholder result to the stack for now
        self.stack_push(StackValue::String(String::new()));
    }

    fn op_builtin(&mut self, env: &Env) {
        assert_eq!(
            compiler::opcode::BUILTIN,
            self.read_u8(),
            "Expected BUILTIN opcode"
        );

        let builtin_idx = self.read_u8();

        if let Some(builtin) = env.builtins.get(builtin_idx as usize) {
            println!("Loading builtin: {:?}", builtin);
            self.stack_push(StackValue::Fn(Box::new(builtin.clone())));

            return;
        }

        panic!("Builtin not found at index: {}", builtin_idx);
    }

    fn op_var(&mut self, env: &Env, vars: &Vec<String>) {
        assert_eq!(compiler::opcode::VAR, self.read_u8(), "Expected VAR opcode");
        let var_idx = self.read_u8();

        if let Some(var) = env.vars.get(var_idx as usize) {
            eprintln!("Loading var: {}", var);
            let value = vars.get(var_idx as usize).expect("undefined variable");

            self.stack_push(StackValue::String(value.clone()));
            return;
        }

        panic!("Variable not found at index: {}", var_idx);
    }

    fn op_prompt(&mut self, env: &Env) {
        assert_eq!(
            compiler::opcode::PROMPT,
            self.read_u8(),
            "Expected PROMPT opcode"
        );

        let prompt_idx = self.read_u8();

        if let Some(prompt) = env.prompts.get(prompt_idx as usize) {
            println!("Loading prompt: {}", prompt);
            self.stack_push(StackValue::String(prompt.clone()));

            return;
        }

        panic!("Prompt not found at index: {}", prompt_idx);
    }

    fn op_secret(&mut self, env: &Env) {
        assert_eq!(
            compiler::opcode::SECRET,
            self.read_u8(),
            "Expected SECRET opcode"
        );

        let secret_idx = self.read_u8();

        if let Some(secret) = env.secrets.get(secret_idx as usize) {
            println!("Loading secret: {}", secret);
            self.stack_push(StackValue::String(secret.clone()));
            return;
        }

        panic!("Secret not found at index: {}", secret_idx);
    }

    fn stack_push(&mut self, value: StackValue) {
        eprintln!("Pushing value: {:?}", value);

        self.stack.push(value);
    }

    fn stack_pop(&mut self) -> StackValue {
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

        self.bytecode.expect("should have bytecode").codes[current_ip as usize]
    }
}
