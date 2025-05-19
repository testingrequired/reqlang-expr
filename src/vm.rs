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
    ) -> Result<(), ()> {
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

        Ok(())
    }

    fn interpret_op(&mut self, env: &Env, runtime_env: &RuntimeEnv, op_code: u8) {
        match op_code {
            compiler::opcode::CALL => self.op_call(env),
            compiler::opcode::BUILTIN => self.op_builtin(env),
            compiler::opcode::VAR => self.op_var(env, &runtime_env.vars),
            compiler::opcode::PROMPT => self.op_prompt(env),
            compiler::opcode::SECRET => self.op_secret(env),
            _ => panic!("Invalid OP code: {op_code}"),
        }
    }

    fn op_call(&mut self, env: &Env) {
        // The following op is BUILTIN
        assert_eq!(self.next_op(), compiler::opcode::BUILTIN);

        // Get the built-in index
        let index = self.next_op();

        // Fetch the built-in function from the environment using the index
        if let Some(builtin) = env.builtins.get(index as usize) {
            println!("Calling built-in function: {:?}", builtin);

            // Next, get the argument count
            let arg_count = self.next_op();
            println!("Number of arguments: {}", arg_count);

            // Retrieve arguments from the stack
            let mut args = Vec::with_capacity(arg_count as usize);
            for _ in 0..arg_count {
                let arg = self.stack_pop();
                args.push(arg);
            }

            // Simulate the function call with the arguments
            println!("Executing call with arguments: {:?}", args);

            // Here you might execute or just simulate the function's logic using args
        }
    }

    fn op_builtin(&mut self, env: &Env) {
        let index = self.next_op();
        if let Some(builtin) = env.builtins.get(index as usize) {
            println!("Loading builtin: {:?}", builtin);
            self.stack_push(StackValue::Fn(Box::new(builtin.clone())));

            return;
        }

        panic!("Builtin not found at index: {}", index);
    }

    fn op_var(&mut self, env: &Env, vars: &Vec<String>) {
        let index = self.next_op();
        if let Some(var) = env.vars.get(index as usize) {
            println!("Loading var: {}", var);
            let value = vars.get(index as usize).expect("undefined variable");

            self.stack_push(StackValue::String(value.clone()));

            return;
        }

        panic!("Variable not found at index: {}", index);
    }

    fn op_prompt(&mut self, env: &Env) {
        let index = self.next_op();
        if let Some(prompt) = env.prompts.get(index as usize) {
            println!("Loading prompt: {}", prompt);
            self.stack_push(StackValue::String(prompt.clone()));

            return;
        }

        panic!("Prompt not found at index: {}", index);
    }

    fn op_secret(&mut self, env: &Env) {
        let index = self.next_op();
        if let Some(secret) = env.secrets.get(index as usize) {
            println!("Loading secret: {}", secret);
            self.stack_push(StackValue::String(secret.clone()));

            return;
        }

        panic!("Secret not found at index: {}", index);
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

    fn next_op(&mut self) -> u8 {
        let current_op = self.ip as u8;

        self.ip += 1;

        current_op
    }
}
