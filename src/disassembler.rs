use crate::compiler::{self, ExprByteCode};

pub struct Disassembler<'bytecode, 'env> {
    bytecode: &'bytecode ExprByteCode,
    env: &'env compiler::Env,
}

impl<'bytecode, 'env> Disassembler<'bytecode, 'env> {
    pub fn new(bytecode: &'bytecode ExprByteCode, env: &'env compiler::Env) -> Self {
        Self { bytecode, env }
    }

    /// Visualize the byte code as text
    pub fn disassemble(&self, level: Option<usize>) -> String {
        let level = level.unwrap_or(0);

        let mut out = String::new();

        let mut op_idx = 0;

        while op_idx < self.bytecode.codes.len() {
            let (op_byte_size, disassembled_byte_idx, disassembled_op) =
                self.disassemble_op(op_idx, level);

            let spacer = if level == 0 {
                String::new()
            } else {
                let s = if level == 1 { "" } else { " " };
                format!(
                    "{space:width$}{bar} ",
                    space = s,
                    bar = "|",
                    width = (level - 1) * 2
                )
            };

            let op_string = &format!("{spacer}{disassembled_byte_idx} {disassembled_op}");
            out.push_str(op_string);

            op_idx += op_byte_size;
        }

        out
    }

    pub fn disassemble_op(&self, op_idx: usize, level: usize) -> (usize, String, String) {
        let op_idx_str = format!("{op_idx:04}");

        let _spacer = if level == 0 {
            String::new()
        } else {
            let s = if level == 1 { "" } else { " " };
            format!(
                "{space:width$}{bar} ",
                space = s,
                bar = "|",
                width = (level - 1) * 2
            )
        };

        let (op_idx_inc, op_str): (usize, String) = match self.bytecode.codes[op_idx] {
            compiler::opcode::VAR => self.disassemble_op_var("VAR", op_idx),
            compiler::opcode::PROMPT => self.disassemble_op_prompt("PROMPT", op_idx),
            compiler::opcode::SECRET => self.disassemble_op_secret("SECRET", op_idx),
            compiler::opcode::BUILTIN => self.disassemble_op_builtin("BUILTIN", op_idx),
            compiler::opcode::CALL => self.disassemble_op_call("CALL", op_idx),
            _ => (1, "".to_string()),
        };

        (op_idx_inc, op_idx_str, op_str)
    }

    fn disassemble_op_var(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_idx = self.bytecode.codes[op_idx + 1];
        let var = &self.env.vars[constant_idx as usize];
        let string = format!("{name:16} {constant_idx:>4} == '{var}'\n");

        (2, string)
    }

    fn disassemble_op_prompt(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_idx = self.bytecode.codes[op_idx + 1];
        let var = &self.env.prompts[constant_idx as usize];
        let string = format!("{name:16} {constant_idx:>4} == '{var}'\n");

        (2, string)
    }

    fn disassemble_op_secret(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_idx = self.bytecode.codes[op_idx + 1];
        let var = &self.env.secrets[constant_idx as usize];
        let string = format!("{name:16} {constant_idx:>4} == '{var}'\n");

        (2, string)
    }

    fn disassemble_op_builtin(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_idx = self.bytecode.codes[op_idx + 1];
        let builtin = &self.env.builtins[constant_idx as usize];
        let string = format!("{name:16} {constant_idx:>4} == '{}'\n", builtin.name);

        (2, string)
    }

    fn disassemble_op_call(&self, name: &str, op_idx: usize) -> (usize, String) {
        let call_op = self.bytecode.codes[op_idx];
        assert_eq!(call_op, compiler::opcode::CALL);

        let builtin_op = self.bytecode.codes[op_idx + 1];
        assert_eq!(builtin_op, compiler::opcode::BUILTIN);

        let builtin_idx = self.bytecode.codes[op_idx + 2];
        let builtin_fn = &self.env.builtins[builtin_idx as usize];

        let arg_count = self.bytecode.codes[op_idx + 3];

        let string = format!(
            "{name:16} {builtin_idx:>4} == {} ({arg_count} args)\n",
            builtin_fn.name
        );

        (4, string)
    }
}
