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

/// Built in functions
pub const BUILTINS: [&str; 4] = ["foo", "bar", "fiz", "baz"];

/// Try to get a builtin function by name
fn get_builtin(identifier: &str) -> Option<u8> {
    BUILTINS
        .into_iter()
        .position(|x| x == identifier)
        .map(|i| i as u8)
}

pub fn compile(expr: &ast::Expr) -> Vec<u8> {
    use opcode::*;

    let mut codes = vec![];

    match expr {
        ast::Expr::Identifier(identifier) => {
            let identifier_name = identifier.0.as_str();

            if let Some(index) = get_builtin(identifier_name) {
                codes.push(BUILTIN);
                codes.push(index as u8);
            } else {
                let identifier_prefix = &identifier_name[..1];

                match identifier_prefix {
                    "?" => {
                        codes.push(PROMPT);
                        codes.push(0u8);
                    }
                    "!" => {
                        codes.push(SECRET);
                        codes.push(0u8);
                    }
                    ":" => {
                        codes.push(VAR);
                        codes.push(0u8);
                    }
                    _ => {}
                };
            }
        }
        ast::Expr::Call(expr_call) => {
            codes.push(opcode::CALL);

            codes.extend(compile(&expr_call.callee.0));

            codes.push(expr_call.args.len() as u8);

            for arg in expr_call.args.iter() {
                codes.extend(compile(&arg.0));
            }
        }
    }

    codes
}
