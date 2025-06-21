//! The dissassembler and associated types

use crate::{
    compiler::{CompileTimeEnv, ExprByteCode, opcode},
    prelude::lookup,
};

pub struct Disassembler<'bytecode, 'env> {
    bytecode: &'bytecode ExprByteCode,
    env: &'env CompileTimeEnv,
}

impl<'bytecode, 'env> Disassembler<'bytecode, 'env> {
    pub fn new(bytecode: &'bytecode ExprByteCode, env: &'env CompileTimeEnv) -> Self {
        Self { bytecode, env }
    }

    /// Visualize the byte code as text
    pub fn disassemble(&self) -> String {
        let mut out = String::new();

        let mut op_idx = 0;

        while op_idx < self.bytecode.codes().len() {
            let (op_byte_size, disassembled_byte_idx, disassembled_op) =
                self.disassemble_op(op_idx);

            let op_string = &format!("{disassembled_byte_idx} {disassembled_op}");
            out.push_str(op_string);

            op_idx += op_byte_size;
        }

        out
    }

    pub fn disassemble_op(&self, op_idx: usize) -> (usize, String, String) {
        let op_idx_str = format!("{op_idx:04}");

        let (op_idx_inc, op_str): (usize, String) = match self.bytecode.codes()[op_idx] {
            opcode::GET => self.disassemble_op_get(op_idx),
            opcode::CALL => self.disassemble_op_call("CALL", op_idx),
            opcode::CONSTANT => self.disassemble_op_constant("CONSTANT", op_idx),
            opcode::TRUE => self.disassemble_op_true("TRUE", op_idx),
            opcode::FALSE => self.disassemble_op_false("FALSE", op_idx),
            opcode::NOT => self.disassemble_op_u8("NOT", op_idx, opcode::NOT),
            opcode::EQ => self.disassemble_op_u8("EQ", op_idx, opcode::EQ),
            opcode::TYPE => self.disassemble_op_u8("TYPE", op_idx, opcode::TYPE),
            _ => (1, "".to_string()),
        };

        (op_idx_inc, op_idx_str, op_str)
    }

    fn disassemble_op_u8(&self, name: &str, op_idx: usize, expected: u8) -> (usize, String) {
        let constant_op = self.bytecode.codes()[op_idx];
        assert_eq!(constant_op, expected);

        let string = format!("{name}\n");

        (1, string)
    }

    fn disassemble_op_true(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_op = self.bytecode.codes()[op_idx];
        assert_eq!(constant_op, opcode::TRUE);

        let string = format!("{name}\n");

        (1, string)
    }

    fn disassemble_op_false(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_op = self.bytecode.codes()[op_idx];
        assert_eq!(constant_op, opcode::FALSE);

        let string = format!("{name}\n");

        (1, string)
    }

    fn disassemble_op_constant(&self, name: &str, op_idx: usize) -> (usize, String) {
        let constant_op = self.bytecode.codes()[op_idx];
        assert_eq!(constant_op, opcode::CONSTANT);

        let constant_idx = self.bytecode.codes()[op_idx + 1] as usize;

        let value = self
            .bytecode
            .strings()
            .get(constant_idx)
            .expect("should have string at index");

        let string = format!("{name:16} {constant_idx:>4} == '{value}'\n");

        (2, string)
    }

    fn disassemble_op_get(&self, op_idx: usize) -> (usize, String) {
        let call_op = self.bytecode.codes()[op_idx];
        assert_eq!(call_op, opcode::GET);

        let lookup_type = self.bytecode.codes()[op_idx + 1];
        let constant_idx = self.bytecode.codes()[op_idx + 2] as usize;

        let value = match lookup_type {
            lookup::BUILTIN => {
                let value = self.env.get_builtin(constant_idx).unwrap();
                &value.name
            }
            lookup::USER_BUILTIN => {
                let value = self.env.get_user_builtin(constant_idx).unwrap();
                &value.name
            }
            lookup::VAR => {
                let value = self.env.get_var(constant_idx).unwrap();

                value
            }
            lookup::PROMPT => {
                let value = self.env.get_prompt(constant_idx).unwrap();

                value
            }
            lookup::SECRET => {
                let value = self.env.get_secret(constant_idx).unwrap();

                value
            }
            lookup::CLIENT_CTX => {
                let value = self.env.get_client_context(constant_idx).unwrap();

                value
            }
            _ => panic!("invalid get lookup code: {}", lookup_type),
        };

        let lookup_type_string = match lookup_type {
            lookup::BUILTIN => "BUILTIN",
            lookup::USER_BUILTIN => "USER_BUILTIN",
            lookup::VAR => "VAR",
            lookup::PROMPT => "PROMPT",
            lookup::SECRET => "SECRET",
            lookup::CLIENT_CTX => "CLIENT_CTX",
            _ => panic!("invalid get lookup code: {}", lookup_type),
        };

        let name = "GET";

        let string = format!("{name} {lookup_type_string:12} {constant_idx:>4} == '{value}'\n");

        (3, string)
    }

    fn disassemble_op_call(&self, name: &str, op_idx: usize) -> (usize, String) {
        let call_op = self.bytecode.codes()[op_idx];
        assert_eq!(call_op, opcode::CALL);

        let arg_count = self.bytecode.codes()[op_idx + 1];

        let string = format!("{name:16} ({arg_count} args)\n",);

        (2, string)
    }
}
