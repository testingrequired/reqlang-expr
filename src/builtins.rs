use core::fmt;
use std::fmt::Display;

use crate::{errors::ExprResult, types::Type, value::Value};

#[derive(Clone)]
pub struct FnArg {
    pub name: &'static str,
    pub ty: Type,
    pub variadic: bool,
}

impl FnArg {
    pub fn new(name: &'static str, ty: Type) -> Self {
        Self {
            name,
            ty,
            variadic: false,
        }
    }

    pub fn new_varadic(name: &'static str, ty: Type) -> Self {
        Self {
            name,
            ty,
            variadic: true,
        }
    }
}

#[derive(Clone)]
/// Builtin function used in expressions
pub struct BuiltinFn<'a> {
    /// Needs to follow identifier naming rules
    pub name: &'static str,
    /// Arguments the function expects
    pub args: &'a [FnArg],
    /// Type returned by the function
    pub return_type: Type,
    /// Function used at runtime
    pub func: fn(Vec<Value>) -> ExprResult<Value>,
}

impl<'a> BuiltinFn<'a> {
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

    /// The default set of builtin functions
    ///
    /// This also defines the lookup index for builtins during compilation
    pub const DEFAULT_BUILTINS: [BuiltinFn<'a>; 17] = [
        BuiltinFn::ID,
        BuiltinFn::NOOP,
        BuiltinFn::IS_EMPTY,
        BuiltinFn::AND,
        BuiltinFn::OR,
        BuiltinFn::COND,
        BuiltinFn::TO_STR,
        BuiltinFn::CONCAT,
        BuiltinFn::CONTAINS,
        BuiltinFn::TRIM,
        BuiltinFn::TRIM_START,
        BuiltinFn::TRIM_END,
        BuiltinFn::LOWERCASE,
        BuiltinFn::UPPERCASE,
        BuiltinFn::TYPE,
        BuiltinFn::EQ,
        BuiltinFn::NOT,
    ];

    // Builtin Definitions

    /// Return the [`Value`] passed in
    pub const ID: BuiltinFn<'static> = BuiltinFn {
        name: "id",
        args: &[FnArg {
            name: "value",
            ty: Type::Value,
            variadic: false,
        }],
        return_type: Type::Value,
        func: Self::id,
    };

    fn id(args: Vec<Value>) -> ExprResult<Value> {
        let arg = args.first().unwrap();

        Ok(arg.clone())
    }

    /// Return the string `` `noop` ``
    pub const NOOP: BuiltinFn<'static> = BuiltinFn {
        name: "noop",
        args: &[],
        return_type: Type::String,
        func: Self::noop,
    };

    fn noop(_: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String(String::from("noop")))
    }

    pub const IS_EMPTY: BuiltinFn<'static> = BuiltinFn {
        name: "is_empty",
        args: &[FnArg {
            name: "value",
            ty: Type::String,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::is_empty,
    };

    fn is_empty(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::Bool(string_arg.is_empty()))
    }

    pub const AND: BuiltinFn<'static> = BuiltinFn {
        name: "and",
        args: &[
            FnArg {
                name: "a",
                ty: Type::Bool,
                variadic: false,
            },
            FnArg {
                name: "b",
                ty: Type::Bool,
                variadic: false,
            },
        ],
        return_type: Type::Bool,
        func: Self::and,
    };

    fn and(args: Vec<Value>) -> ExprResult<Value> {
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

    pub const OR: BuiltinFn<'static> = BuiltinFn {
        name: "or",
        args: &[
            FnArg {
                name: "a",
                ty: Type::Bool,
                variadic: false,
            },
            FnArg {
                name: "b",
                ty: Type::Bool,
                variadic: false,
            },
        ],
        return_type: Type::Bool,
        func: Self::or,
    };

    fn or(args: Vec<Value>) -> ExprResult<Value> {
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

    pub const COND: BuiltinFn<'static> = BuiltinFn {
        name: "cond",
        args: &[
            FnArg {
                name: "cond",
                ty: Type::Bool,
                variadic: false,
            },
            FnArg {
                name: "then",
                ty: Type::Value,
                variadic: false,
            },
            FnArg {
                name: "else",
                ty: Type::Value,
                variadic: false,
            },
        ],
        return_type: Type::Bool,
        func: Self::cond,
    };

    fn cond(args: Vec<Value>) -> ExprResult<Value> {
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

    pub const TO_STR: BuiltinFn<'static> = BuiltinFn {
        name: "to_str",
        args: &[FnArg {
            name: "value",
            ty: Type::Value,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::to_str,
    };

    fn to_str(args: Vec<Value>) -> ExprResult<Value> {
        let value_arg = args.first().expect("should have string expression passed");

        Ok(match value_arg {
            Value::String(_) => value_arg.clone(),
            _ => Value::String(value_arg.to_string()),
        })
    }

    pub const CONCAT: BuiltinFn<'static> = BuiltinFn {
        name: "concat",
        args: &[
            FnArg {
                name: "a",
                ty: Type::Value,
                variadic: false,
            },
            FnArg {
                name: "b",
                ty: Type::Value,
                variadic: false,
            },
            FnArg {
                name: "rest",
                ty: Type::Value,
                variadic: true,
            },
        ],
        return_type: Type::String,
        func: Self::concat,
    };

    fn concat(args: Vec<Value>) -> ExprResult<Value> {
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

    pub const CONTAINS: BuiltinFn<'static> = BuiltinFn {
        name: "contains",
        args: &[
            FnArg {
                name: "needle",
                ty: Type::String,
                variadic: false,
            },
            FnArg {
                name: "haystack",
                ty: Type::String,
                variadic: false,
            },
        ],
        return_type: Type::Bool,
        func: Self::contains,
    };

    fn contains(args: Vec<Value>) -> ExprResult<Value> {
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

    pub const TRIM: BuiltinFn<'static> = BuiltinFn {
        name: "trim",
        args: &[FnArg {
            name: "value",
            ty: Type::String,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::trim,
    };

    fn trim(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.trim().to_string()))
    }

    pub const TRIM_START: BuiltinFn<'static> = BuiltinFn {
        name: "trim_start",
        args: &[FnArg {
            name: "value",
            ty: Type::String,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::trim_start,
    };

    fn trim_start(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.trim_start().to_string()))
    }

    pub const TRIM_END: BuiltinFn<'static> = BuiltinFn {
        name: "trim_end",
        args: &[FnArg {
            name: "value",
            ty: Type::String,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::trim_end,
    };

    fn trim_end(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.trim_end().to_string()))
    }

    pub const LOWERCASE: BuiltinFn<'static> = BuiltinFn {
        name: "lowercase",
        args: &[{
            FnArg {
                name: "value",
                ty: Type::String,
                variadic: false,
            }
        }],
        return_type: Type::String,
        func: Self::lowercase,
    };

    fn lowercase(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.to_lowercase().to_string()))
    }

    pub const UPPERCASE: BuiltinFn<'static> = BuiltinFn {
        name: "uppercase",
        args: &[FnArg {
            name: "value",
            ty: Type::String,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::uppercase,
    };

    fn uppercase(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::String(string_arg.to_uppercase().to_string()))
    }

    pub const TYPE: BuiltinFn<'static> = BuiltinFn {
        name: "type",
        args: &[FnArg {
            name: "value",
            ty: Type::Value,
            variadic: false,
        }],
        return_type: Type::String,
        func: Self::get_type,
    };

    fn get_type(args: Vec<Value>) -> ExprResult<Value> {
        let value_arg = args.first().expect("should have first expression passed");

        Ok(Value::Type(Type::Type(value_arg.get_type().into()).into()))
    }

    pub const EQ: BuiltinFn<'static> = BuiltinFn {
        name: "eq",
        args: &[
            {
                FnArg {
                    name: "a",
                    ty: Type::Value,
                    variadic: false,
                }
            },
            {
                FnArg {
                    name: "b",
                    ty: Type::Value,
                    variadic: false,
                }
            },
        ],
        return_type: Type::Bool,
        func: Self::eq,
    };

    fn eq(args: Vec<Value>) -> ExprResult<Value> {
        let first_arg = args.first().expect("should have first expression passed");
        let second_arg = args.get(1).expect("should have second expression passed");

        let equals = first_arg == second_arg;

        Ok(equals.into())
    }

    pub const NOT: BuiltinFn<'static> = BuiltinFn {
        name: "not",
        args: &[{
            let ty = Type::Bool;
            FnArg {
                name: "value",
                ty,
                variadic: false,
            }
        }],
        return_type: Type::Bool,
        func: Self::not,
    };

    fn not(args: Vec<Value>) -> ExprResult<Value> {
        let value_arg = args.first().expect("should have first expression passed");

        let value = &value_arg.get_bool()?;

        Ok(Value::Bool(!value))
    }
}

impl<'a> PartialEq for BuiltinFn<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> Display for BuiltinFn<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = &self.name;
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| {
                let prefix: &str = if arg.variadic { "..." } else { "" };

                format!("{prefix}{}: {}", arg.name, arg.ty.name())
            })
            .collect();

        let args: String = args.join(", ");

        let return_type: String = self.return_type.name().to_string();

        write!(f, "{name}({args}) -> {return_type}")
    }
}

impl<'a> fmt::Debug for BuiltinFn<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = &self.name;
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| {
                let prefix: &str = if arg.variadic { "..." } else { "" };

                format!("{prefix}{}: {}", arg.name, arg.ty.name())
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

#[cfg(test)]
mod value_tests {
    use super::*;

    fn example_builtin(_args: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String("".to_string()))
    }

    #[test]
    fn test_builtins_display_var_arity() {
        let f = BuiltinFn {
            name: "test_builtin",
            args: &[FnArg::new_varadic("rest", Type::String)],
            return_type: Type::String,
            func: example_builtin,
        };
        assert_eq!("test_builtin(...rest: String) -> String", format!("{}", f))
    }

    #[test]
    fn test_builtins_display_0_arity() {
        assert_eq!(
            "test_builtin() -> String",
            format!(
                "{}",
                BuiltinFn {
                    name: "test_builtin",
                    args: &[],
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
                    name: "test_builtin",
                    args: &[],
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
                    name: "test_builtin",
                    args: &[FnArg::new("value", Type::String)],
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
                    name: "test_builtin",
                    args: &[FnArg::new("value", Type::String)],
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
                    name: "test_builtin",
                    args: &[FnArg::new("a", Type::String), FnArg::new("b", Type::String)],
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
                    name: "test_builtin",
                    args: &[FnArg::new("a", Type::String), FnArg::new("b", Type::String)],
                    return_type: Type::String,
                    func: example_builtin
                }
            )
        )
    }
}
