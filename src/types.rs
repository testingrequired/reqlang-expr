use std::fmt::{Debug, Display};

use regex::Regex;

use crate::{prelude::BuiltinFn, value::Value};

#[derive(Clone, PartialEq)]
pub enum Type {
    Value,
    String,
    Fn {
        args: Vec<Type>,
        variadic_arg: Option<Box<Type>>,
        returns: Box<Type>,
    },
    Bool,
    Type(Box<Type>),
    Unknown,
}

impl Type {
    pub fn from(name: &str) -> Self {
        match name {
            "String" => Type::String,
            "Bool" => Type::Bool,
            "Value" => Type::Value,
            _ => {
                if let Some(captures) = Regex::new(r"^Type<(.+)>$").unwrap().captures(name) {
                    return Type::Type(Type::from(&captures[1]).into());
                }

                if let Some(captures) = Regex::new(
                    r"^Fn\((?P<args>(?:\w+(?:,\s*)?)*)\s*(?:\.\.\.(?P<varg>\w+))?\)\s*->\s*(?P<return>\w+)$",
                ).unwrap().captures(name) {
                    let args_str = captures.name("args").map_or("", |m| m.as_str());
                    let variadic_str = captures.name("varg").map_or("", |m| m.as_str());
                    let return_type_str = captures.name("return").unwrap().as_str();

                    let args: Vec<Type> = if args_str.trim().is_empty() {
                        vec![]
                    } else {
                        args_str.split(',')
                            .filter_map(|s| {
                                let trimmed = s.trim();
                                if trimmed.is_empty() { None } else { Some(Type::from(trimmed)) }
                            })
                            .collect()
                    };

                    let variadic_arg = if variadic_str.is_empty() {
                        None
                    } else {
                        Some(Type::from(variadic_str).into())
                    };

                    let returns = Type::from(return_type_str).into();

                    return Type::Fn {
                        args,
                        variadic_arg,
                        returns,
                    };
                }

                Type::Unknown
            }
        }
    }

    pub fn name(&self) -> String {
        match self {
            Type::Value => "Value".to_string(),
            Type::String => "String".to_string(),
            Type::Fn {
                args,
                variadic_arg,
                returns,
            } => {
                let mut args: Vec<String> = args.iter().map(|arg| arg.name()).collect();

                if let Some(varg) = variadic_arg {
                    args.push(format!("...{}", varg.name()));
                }

                let args = args.join(", ");

                let returns = returns.name();

                format!("Fn({args}) -> {returns}")
            }
            Type::Bool => "Bool".to_string(),
            Type::Type(ty) => format!("{}", ty.name()),
            Type::Unknown => "Unknown".to_string(),
        }
    }

    pub fn is_type(&self) -> bool {
        match self {
            Type::Type(_) => true,
            _ => false,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<Value> for Type {
    fn from(value: Value) -> Self {
        match value {
            Value::String(_) => Type::String,
            Value::Fn(builtin_fn) => {
                let mut args: Vec<Type> =
                    builtin_fn.args.iter().map(|arg| arg.ty.clone()).collect();

                let variadic_arg = builtin_fn
                    .args
                    .last()
                    .filter(|arg| arg.variadic)
                    .map(|arg| Box::new(arg.ty.clone()));

                if variadic_arg.is_some() {
                    args.pop();
                }

                Type::Fn {
                    args,
                    variadic_arg,
                    returns: Box::new(builtin_fn.return_type.clone()),
                }
            }
            Value::Bool(_) => Type::Bool,
            Value::Type(ty) => *ty.clone(),
        }
    }
}

impl From<BuiltinFn<'static>> for Type {
    fn from(value: BuiltinFn) -> Self {
        let args: Vec<Type> = value
            .args
            .iter()
            .filter(|x| x.variadic == false)
            .map(|x| x.ty.clone())
            .collect();
        let varg = value
            .args
            .iter()
            .find(|x| x.variadic == true)
            .map(|x| Box::new(x.ty.clone()));
        let returns = value.return_type.clone();

        Self::Fn {
            args,
            variadic_arg: varg,
            returns: returns.into(),
        }
    }
}

#[cfg(test)]
mod from_tests {
    use crate::prelude::BuiltinFn;

    use super::*;

    #[test]
    fn test_from_type_value() {
        let type_value = Value::Type(Type::String.into());
        let ty: Type = type_value.into();
        assert_eq!(Type::String, ty);
    }

    #[test]
    fn test_get_type_type_value() {
        let string_value = Value::Type(Type::String.into());
        let ty: Type = string_value.get_type();
        assert_eq!(Type::String, ty);
    }

    #[test]
    fn test_from_string_value() {
        let string_value = Value::String("test".to_string());
        let ty: Type = string_value.into();
        assert_eq!(Type::String, ty);
    }

    #[test]
    fn test_get_type_string_value() {
        let string_value = Value::String("test".to_string());
        let ty: Type = string_value.get_type();
        assert_eq!(Type::String, ty);
    }

    #[test]
    fn test_from_bool_value() {
        let bool_value = Value::Bool(true);
        let ty: Type = bool_value.into();
        assert_eq!(Type::Bool, ty);
    }

    #[test]
    fn test_get_type_bool_value() {
        let bool_value = Value::Bool(true);
        let ty: Type = bool_value.get_type();
        assert_eq!(Type::Bool, ty);
    }

    #[test]
    fn test_from_fn_value() {
        let builtin_fn = Value::Fn(BuiltinFn::ID.into());

        let ty: Type = builtin_fn.into();

        assert_eq!(
            Type::Fn {
                args: vec![Type::Value],
                variadic_arg: None,
                returns: Box::new(Type::Value),
            },
            ty
        );
    }

    #[test]
    fn test_get_type_fn_value() {
        let builtin_fn = Value::Fn(BuiltinFn::ID.into());

        let ty: Type = builtin_fn.get_type();

        assert_eq!(
            Type::Fn {
                args: vec![Type::Value],
                variadic_arg: None,
                returns: Box::new(Type::Value),
            },
            ty
        );
    }
}

#[cfg(test)]
mod name_and_display_tests {
    use super::*;

    #[test]
    fn test_name_value() {
        assert_eq!("Value", Type::Value.name());
    }

    #[test]
    fn test_name_string() {
        assert_eq!("String", Type::String.name());
    }

    #[test]
    fn test_name_fn_0_args() {
        assert_eq!(
            "Fn() -> String",
            Type::Fn {
                args: vec![],
                variadic_arg: None,
                returns: Type::String.into(),
            }
            .name()
        );
    }

    #[test]
    fn test_name_fn_0_args_and_varadic() {
        assert_eq!(
            "Fn(...Value) -> String",
            Type::Fn {
                args: vec![],
                variadic_arg: Some(Type::Value.into()),
                returns: Type::String.into(),
            }
            .name()
        );
    }

    #[test]
    fn test_name_fn_1_args() {
        assert_eq!(
            "Fn(Value) -> String",
            Type::Fn {
                args: vec![Type::Value],
                variadic_arg: None,
                returns: Type::String.into(),
            }
            .name()
        );
    }

    #[test]
    fn test_name_fn_2_args() {
        assert_eq!(
            "Fn(Value, String) -> String",
            Type::Fn {
                args: vec![Type::Value, Type::String],
                variadic_arg: None,
                returns: Type::String.into(),
            }
            .name()
        );
    }

    #[test]
    fn test_name_fn_2_args_and_varadic() {
        assert_eq!(
            "Fn(Value, String, ...Value) -> String",
            Type::Fn {
                args: vec![Type::Value, Type::String],
                variadic_arg: Some(Type::Value.into()),
                returns: Type::String.into(),
            }
            .name()
        );
    }

    #[test]
    fn test_name_bool() {
        assert_eq!("Bool", Type::Bool.name());
    }
}

#[cfg(test)]
mod from_string_tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn from_string_to_string() {
        let ty = Type::from("String");

        assert_eq!(Type::String, ty);
    }

    #[test]
    fn from_string_to_bool() {
        let ty = Type::from("Bool");

        assert_eq!(Type::Bool, ty);
    }

    #[test]
    fn from_string_to_value() {
        let ty = Type::from("Value");

        assert_eq!(Type::Value, ty);
    }

    #[test]
    fn from_string_to_unknown() {
        let ty = Type::from("Unknown");

        assert_eq!(Type::Unknown, ty);
    }

    #[test]
    fn from_string_to_type_string() {
        let ty = Type::from("Type<String>");

        assert_eq!(Type::Type(Type::String.into()), ty);
    }

    #[test]
    fn from_string_to_type_fn_zero_args() {
        let ty = Type::from("Fn() -> Value");

        assert_eq!(
            Type::Fn {
                args: vec![],
                variadic_arg: None,
                returns: Type::Value.into()
            },
            ty
        );
    }

    #[test]
    fn from_string_to_type_fn_single_arg() {
        let ty = Type::from("Fn(String) -> Value");

        assert_eq!(
            Type::Fn {
                args: vec![Type::String],
                variadic_arg: None,
                returns: Type::Value.into()
            },
            ty
        );
    }

    #[test]
    fn from_string_to_type_fn_single_varg() {
        let ty = Type::from("Fn(...String) -> Value");

        assert_eq!(
            Type::Fn {
                args: vec![],
                variadic_arg: Some(Type::String.into()),
                returns: Type::Value.into()
            },
            ty
        );
    }

    #[test]
    fn from_string_to_type_fn_multiple_args() {
        let ty = Type::from("Fn(String, Bool, String) -> String");

        assert_eq!(
            Type::Fn {
                args: vec![Type::String, Type::Bool, Type::String],
                variadic_arg: None,
                returns: Type::String.into()
            },
            ty
        );
    }

    #[test]
    fn from_string_to_type_fn_multiple_args_and_varg() {
        let ty = Type::from("Fn(String, Bool, String, ...Value) -> String");

        assert_eq!(
            Type::Fn {
                args: vec![Type::String, Type::Bool, Type::String],
                variadic_arg: Some(Type::Value.into()),
                returns: Type::String.into()
            },
            ty
        );
    }
}
