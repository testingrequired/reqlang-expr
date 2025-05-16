use crate::ast;

pub mod opcode {
    iota::iota! {
        pub const
        CALL: u8 = iota;,
        BUILTIN,
        VAR,
        PROMPT,
        SECRET
    }
}

/// Try to get a string from a list
fn get(list: &Vec<String>, identifier: &str) -> Option<u8> {
    list.into_iter()
        .position(|x| x == identifier)
        .map(|i| i as u8)
}

#[derive(Debug, Clone)]
pub struct Fn {
    pub name: String,
    pub arity: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    pub builtins: Vec<Fn>,
    pub vars: Vec<String>,
    pub prompts: Vec<String>,
    pub secrets: Vec<String>,
}

impl Env {
    pub fn get_builtin(&self, name: &str) -> Option<(&Fn, u8)> {
        let index = self.builtins.iter().position(|x| x.name == name);

        let result = index.map(|i| (self.builtins.get(i).unwrap(), i as u8));
        result
    }
}

pub fn compile(expr: &ast::Expr, env: &Env) -> Vec<u8> {
    use opcode::*;

    let mut codes = vec![];

    match expr {
        ast::Expr::Identifier(identifier) => {
            let identifier_name = identifier.0.as_str();

            if let Some((_, index)) = env.get_builtin(identifier_name) {
                codes.push(BUILTIN);
                codes.push(index as u8);
            } else {
                let identifier_prefix = &identifier_name[..1];
                let identifier_suffix = &identifier_name[1..];

                match identifier_prefix {
                    "?" => {
                        if let Some(index) = get(&env.prompts, identifier_suffix) {
                            codes.push(PROMPT);
                            codes.push(index);
                        }
                    }
                    "!" => {
                        if let Some(index) = get(&env.secrets, identifier_suffix) {
                            codes.push(SECRET);
                            codes.push(index);
                        }
                    }
                    ":" => {
                        if let Some(index) = get(&env.vars, identifier_suffix) {
                            codes.push(VAR);
                            codes.push(index);
                        }
                    }
                    _ => {}
                };
            }
        }
        ast::Expr::Call(expr_call) => {
            codes.push(opcode::CALL);

            codes.extend(compile(&expr_call.callee.0, env));

            codes.push(expr_call.args.len() as u8);

            for arg in expr_call.args.iter() {
                codes.extend(compile(&arg.0, env));
            }
        }
    }

    codes
}
