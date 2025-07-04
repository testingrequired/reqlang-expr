use core::fmt;
use std::fmt::Display;

use crate::{errors::ExprResult, types::Type, value::Value};

#[derive(Clone)]
pub struct FnArg {
    pub name: String,
    pub ty: Type,
    pub variadic: bool,
}

impl FnArg {
    pub fn new(name: &str, ty: Type) -> Self {
        Self {
            name: String::from(name),
            ty,
            variadic: false,
        }
    }

    pub fn new_varadic(name: &str, ty: Type) -> Self {
        Self {
            name: String::from(name),
            ty,
            variadic: true,
        }
    }
}

#[derive(Clone)]
/// Builtin function used in expressions
pub struct BuiltinFn {
    // Needs to follow identifier naming rules
    pub name: String,
    // Arguments the function expects
    pub args: Vec<FnArg>,
    pub return_type: Type,
    // Function used at runtime
    pub func: fn(Vec<Value>) -> ExprResult<Value>,
}

impl BuiltinFn {
    pub fn arity(&self) -> u8 {
        let len = self.args.len() as u8;

        if self.is_variadic() { len - 1 } else { len }
    }

    pub fn is_variadic(&self) -> bool {
        self.args.last().map(|arg| arg.variadic).unwrap_or(false)
    }

    pub fn arity_matches(&self, arity: u8) -> bool {
        if self.is_variadic() {
            self.arity() <= arity
        } else {
            self.arity() == arity
        }
    }
}

impl PartialEq for BuiltinFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = &self.name;
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| {
                let prefix: &str = if arg.variadic { "..." } else { "" };

                format!("{prefix}{}: {}", arg.name.clone(), arg.ty.name())
            })
            .collect();

        let args: String = args.join(", ");

        let return_type: String = self.return_type.name().to_string();

        write!(f, "{name}({args}) -> {return_type}")
    }
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = &self.name;
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| {
                let prefix: &str = if arg.variadic { "..." } else { "" };

                format!("{prefix}{}: {}", arg.name.clone(), arg.ty.name())
            })
            .collect();

        let args: String = args.join(", ");

        let return_type: String = self.return_type.name().to_string();

        write!(f, "{name}({args}) -> {return_type}")
    }
}

#[derive(Debug, PartialEq)]
pub enum FnArity {
    N(u8),
    Variadic { n: u8 },
}

pub struct BuiltinFns;

impl BuiltinFns {
    pub fn id(args: Vec<Value>) -> ExprResult<Value> {
        let arg = args.first().unwrap();

        Ok(arg.clone())
    }

    pub fn noop(_: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String(String::from("noop")))
    }

    pub fn is_empty(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::Bool(string_arg.is_empty()))
    }

    pub fn and(args: Vec<Value>) -> ExprResult<Value> {
        let a_arg = args
            .first()
            .expect("should have first expression passed")
            .get_bool()?;
        let b_arg = args
            .get(1)
            .expect("should have second expression passed")
            .get_bool()?;

        Ok(Value::Bool(a_arg && b_arg))
    }

    pub fn or(args: Vec<Value>) -> ExprResult<Value> {
        let a_arg = args
            .first()
            .expect("should have first expression passed")
            .get_bool()?;
        let b_arg = args
            .get(1)
            .expect("should have second expression passed")
            .get_bool()?;

        Ok(Value::Bool(a_arg || b_arg))
    }

    pub fn cond(args: Vec<Value>) -> ExprResult<Value> {
        let cond_arg = args
            .first()
            .expect("should have cond expression passed")
            .get_bool()?;
        let then_arg = args
            .get(1)
            .cloned()
            .expect("should have then expression passed");
        let else_arg = args
            .get(2)
            .cloned()
            .expect("should have else expression passed");

        if cond_arg { Ok(then_arg) } else { Ok(else_arg) }
    }

    pub fn to_str(args: Vec<Value>) -> ExprResult<Value> {
        let value_arg = args.first().expect("should have string expression passed");

        Ok(match value_arg {
            Value::String(_) => value_arg.clone(),
            _ => Value::String(value_arg.to_string()),
        })
    }

    pub fn concat(args: Vec<Value>) -> ExprResult<Value> {
        let mut result = String::new();

        for arg in args {
            let value = match arg {
                Value::String(string) => string,
                _ => arg.to_string(),
            };

            result.push_str(value.as_str());
        }

        Ok(Value::String(result))
    }

    pub fn contains(args: Vec<Value>) -> ExprResult<Value> {
        let needle_arg = args
            .first()
            .expect("should have first expression passed")
            .get_string()?;
        let haystack_arg = args
            .get(1)
            .expect("should have second expression passed")
            .get_string()?;

        Ok(Value::Bool(haystack_arg.contains(needle_arg)))
    }

    pub fn trim(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.trim().to_string()))
    }

    pub fn trim_start(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.trim_start().to_string()))
    }

    pub fn trim_end(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.trim_end().to_string()))
    }

    pub fn lowercase(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.to_lowercase().to_string()))
    }

    pub fn uppercase(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.to_uppercase().to_string()))
    }

    pub fn get_type(args: Vec<Value>) -> ExprResult<Value> {
        let value_arg = args.first().expect("should have first expression passed");

        Ok(Value::Type(value_arg.get_type().into()))
    }

    pub fn eq(args: Vec<Value>) -> ExprResult<Value> {
        let first_arg = args.first().expect("should have first expression passed");
        let second_arg = args.get(1).expect("should have second expression passed");

        let equals = first_arg == second_arg;

        Ok(equals.into())
    }

    pub fn not(args: Vec<Value>) -> ExprResult<Value> {
        let value_arg = args.first().expect("should have first expression passed");

        let value = &value_arg.get_bool()?;

        Ok(Value::Bool(!value))
    }
}

#[cfg(test)]
mod value_tests {
    use super::*;

    fn example_builtin(_args: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String("".to_string()))
    }

    #[test]
    fn test_builtins_display_var_arity() {
        assert_eq!(
            "test_builtin(...rest: String) -> String",
            format!(
                "{}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![FnArg::new_varadic("rest", Type::String)],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }

    #[test]
    fn test_builtins_display_0_arity() {
        assert_eq!(
            "test_builtin() -> String",
            format!(
                "{}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }

    #[test]
    fn test_builtins_debug_0_arity() {
        assert_eq!(
            "test_builtin() -> String",
            format!(
                "{:#?}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }

    #[test]
    fn test_builtins_display_1_arity() {
        assert_eq!(
            "test_builtin(value: String) -> String",
            format!(
                "{}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }

    #[test]
    fn test_builtins_debug_1_arity() {
        assert_eq!(
            "test_builtin(value: String) -> String",
            format!(
                "{:#?}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![FnArg::new("value", Type::String)],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }

    #[test]
    fn test_builtins_display_2_arity() {
        assert_eq!(
            "test_builtin(a: String, b: String) -> String",
            format!(
                "{}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![FnArg::new("a", Type::String), FnArg::new("b", Type::String)],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }

    #[test]
    fn test_builtins_debug_2_arity() {
        assert_eq!(
            "test_builtin(a: String, b: String) -> String",
            format!(
                "{:#?}",
                BuiltinFn {
                    name: "test_builtin".to_string(),
                    args: vec![FnArg::new("a", Type::String), FnArg::new("b", Type::String)],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }
}
