use crate::compiler::ExprByteCode;

#[derive(Debug, Default)]
pub struct Vm<'bytecode> {
    bytecode: Option<&'bytecode ExprByteCode>,
    ip: usize,
}

impl<'bytecode> Vm<'bytecode> {
    pub fn interpret(&mut self, bytecode: &'bytecode ExprByteCode) -> Result<(), ()> {
        self.bytecode = Some(bytecode);
        self.ip = 0;

        while let Some(op_code) = self
            .bytecode
            .and_then(|ExprByteCode { codes }| codes.get(self.ip))
        {
            self.ip += 1;

            match op_code {
                _ => {}
            }
        }

        Ok(())
    }
}
