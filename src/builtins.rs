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
    pub const DEFAULT_BUILTINS: [BuiltinFn<'a>; 18] = [
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
        BuiltinFn::JSONOBJ,
    ];

    // Builtin Definitions

    /// Return [`Value`] passed in
    ///
    /// `(id :variable)`
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

    /// Return [`Value::String`] of `` `noop` ``
    ///
    /// `(noop)`
    pub const NOOP: BuiltinFn<'static> = BuiltinFn {
        name: "noop",
        args: &[],
        return_type: Type::String,
        func: Self::noop,
    };

    fn noop(_: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String(String::from("noop")))
    }

    /// Return [`Type::Bool`] if [`Value::String`] is empty
    ///
    /// `` (is_empty `...`) ``
    pub const IS_EMPTY: BuiltinFn<'static> = BuiltinFn {
        name: "is_empty",
        args: &[FnArg {
            name: "value",
            ty: Type::String,
            variadic: false,
        }],
        return_type: Type::Bool,
        func: Self::is_empty,
    };

    fn is_empty(args: Vec<Value>) -> ExprResult<Value> {
        let string_arg = args
            .first()
            .expect("should have string expression passed")
            .get_string()?;

        Ok(Value::Bool(string_arg.is_empty()))
    }

    /// Return [`Type::Bool`] if args [`Value::Bool`] are both `true`
    ///
    /// `(and true true)`
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

    /// Return [`Type::Bool`] if at least one [`Value::Bool`] is `true`
    ///
    /// `(or false true)`
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

    /// Return conditional [`Value`] based on if conditional [`Value::Bool`] is true
    ///
    /// `` (cond true `foo` `bar`) ``
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

    /// Return [`Value::String`] for the given [`Value`]
    ///
    /// `(to_str true)`
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

    /// Return [`Value::String`] concatenation of the given [`Value`] arguments
    ///
    /// `` (concat `Hello` `, ` `World!`) ``
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

    /// Returns [`Value::Bool`] if `needle` [`Value::String`] is in `haystack` [`Value::String`]
    ///
    /// `` (contains `Hello` `Hello World`) ``
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

    /// Returns [`Value::String`] with whitespace trimmed from both sides of [`Value::String`]
    ///
    /// `` (trim ` Hello `) ``
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

    /// Returns [`Value::String`] with whitespace trimmed from start of [`Value::String`]
    ///
    /// `` (trim_start ` Hello`) ``
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

    /// Returns [`Value::String`] with whitespace trimmed from end of [`Value::String`]
    ///
    /// `` (trim_end `Hello `) ``
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

    /// Returns [`Value::String`] lowercased
    ///
    /// `` (lowercase ` HELLO`) ``
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

    /// Returns [`Value::String`] uppercased
    ///
    /// `` (uppercase ` HELLO`) ``
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

    /// Returns [`Value::Type`] of [`Value`]
    ///
    /// (type true)
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

    /// Returns [`Value::Bool`] if two [`Value`] are equal
    ///
    /// (eq true true)
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

    /// Returns [`Value::Bool`] negated
    ///
    /// (not true)
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

    /// Get the value from a JSON object by key
    ///
    /// (jsonobj `{"greeting": "Hello", "name": "World"}` `greeting`)
    pub const JSONOBJ: BuiltinFn<'static> = BuiltinFn {
        name: "jsonobj",
        args: &[
            {
                let ty = Type::String;
                FnArg {
                    name: "json_obj",
                    ty,
                    variadic: false,
                }
            },
            {
                let ty = Type::Value;
                FnArg {
                    name: "obj_keys",
                    ty,
                    variadic: true,
                }
            },
        ],
        return_type: Type::Value,
        func: Self::jsonobj,
    };

    fn jsonobj(args: Vec<Value>) -> ExprResult<Value> {
        let json_obj_arg = args.first().expect("should have first expression passed");
        let json_obj_str = json_obj_arg.get_string()?;
        let mut json_obj_value: serde_json::Value =
            serde_json::from_str(json_obj_str).expect("should be json_obj_str as JSON");

        let args = &args[1..];

        for key in args.iter() {
            match key {
                Value::String(key) => {
                    let value = json_obj_value.as_object().unwrap().get(key);

                    if let Some(value) = value {
                        json_obj_value = value.clone();
                    }
                }
                Value::Number(index) => {
                    let value = json_obj_value
                        .as_array()
                        .unwrap()
                        .get(index.clone() as usize);

                    if let Some(value) = value {
                        json_obj_value = value.clone();
                    }
                }
                _ => {}
            }
        }

        Ok(Value::String(
            serde_json::to_string(&json_obj_value)
                .expect("should serialize json_obj_value to JSON"),
        ))
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
