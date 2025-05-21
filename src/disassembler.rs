use crate::{
    compiler::{self, ExprByteCode},
    prelude::lookup,
};

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
            compiler::opcode::GET => self.disassemble_op_get("GET", op_idx),
            compiler::opcode::CALL => self.disassemble_op_call("CALL", op_idx),
            compiler::opcode::CONSTANT => self.disassemble_op_constant("CONSTANT", op_idx),
            _ => (1, "".to_string()),
        };

        (op_idx_inc, op_idx_str, op_str)
    }

    fn disassemble_op_constant(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_op = self.bytecode.codes[op_idx];
        assert_eq!(constant_op, compiler::opcode::CONSTANT);

        let constant_idx = self.bytecode.codes[op_idx + 1] as usize;

        let value = self
            .bytecode
            .strings
            .get(constant_idx)
            .expect("should have string at index");

        let string = format!("{name:16} {constant_idx:>4} == '{value}'\n");

        (2, string)
    }

    fn disassemble_op_get(&self, name: &str, op_idx: usize) -> (usize, String) {
        let call_op = self.bytecode.codes[op_idx];
        assert_eq!(call_op, compiler::opcode::GET);

        let lookup_type = self.bytecode.codes[op_idx + 1];
        let constant_idx = self.bytecode.codes[op_idx + 2] as usize;

        let value = match lookup_type {
            lookup::BUILTIN => {
                let value = self
                    .env
                    .get_builtin(constant_idx)
                    .expect(&format! {"undefined builtin: {constant_idx}"});
                &value.name
            }
            lookup::VAR => {
                let value = self
                    .env
                    .vars
                    .get(constant_idx)
                    .expect(&format! {"undefined variable: {constant_idx}"});

                value
            }
            lookup::PROMPT => {
                let value = self
                    .env
                    .prompts
                    .get(constant_idx)
                    .expect(&format! {"undefined prompt: {constant_idx}"});

                value
            }
            lookup::SECRET => {
                let value = self
                    .env
                    .secrets
                    .get(constant_idx)
                    .expect(&format! {"undefined secret: {constant_idx}"});

                value
            }
            _ => panic!("invalid get lookup code: {}", lookup_type),
        };

        let string = format!("{name:16} {constant_idx:>4} == '{value}'\n");

        (3, string)
    }

    fn disassemble_op_call(&self, name: &str, op_idx: usize) -> (usize, String) {
        let call_op = self.bytecode.codes[op_idx];
        assert_eq!(call_op, compiler::opcode::CALL);

        let arg_count = self.bytecode.codes[op_idx + 1];

        let string = format!("{name:16} ({arg_count} args)\n",);

        (2, string)
    }
}
