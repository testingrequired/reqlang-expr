//! The core value type used in the virtual machine.

use std::fmt::Display;

use crate::{
    builtins::BuiltinFn,
    errors::{ExprResult, RuntimeError},
    types::Type,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Fn(Box<BuiltinFn<'static>>),
    Bool(bool),
    Type(Box<Type>),
}

impl Value {
    pub fn get_type(&self) -> Type {
        self.clone().into()
    }

    pub fn get_string(&self) -> ExprResult<&str> {
        match self {
            Value::String(s) => Ok(s.as_str()),
            _ => Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::String,
                    actual: self.get_type(),
                }
                .into(),
                0..0,
            )]),
        }
    }

    pub fn get_func(&self) -> ExprResult<Box<BuiltinFn>> {
        match self {
            Value::Fn(f) => Ok(f.clone()),
            _ => Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::Fn {
                        args: vec![],
                        variadic_arg: Some(Type::Value.into()),
                        returns: Type::Value.into(),
                    },
                    actual: self.get_type(),
                }
                .into(),
                0..0,
            )]),
        }
    }

    pub fn get_bool(&self) -> ExprResult<bool> {
        match self {
            Value::Bool(s) => Ok(*s),
            _ => Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::Bool,
                    actual: self.get_type(),
                }
                .into(),
                0..0,
            )]),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "`{}`", string),
            Value::Fn(builtin) => write!(f, "{builtin:?}"),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Type(ty) => write!(f, "{}", ty),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn example_builtin(_args: Vec<Value>) -> ExprResult<Value> {
        Ok(Value::String("".to_string()))
    }

    #[test]
    fn get_bool_on_string() {
        assert_eq!(
            Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::Bool,
                    actual: Type::String
                }
                .into(),
                0..0
            )]),
            Value::String("string".to_string()).get_bool()
        );
    }

    #[test]
    fn get_bool_on_bool_true() {
        assert_eq!(Ok(true), Value::Bool(true).get_bool());
    }

    #[test]
    fn get_string_on_bool() {
        assert_eq!(
            Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::String,
                    actual: Type::Bool
                }
                .into(),
                0..0
            )]),
            Value::Bool(true).get_string()
        );
    }

    #[test]
    fn get_string_on_string() {
        let value = Value::String("test".to_string());
        assert_eq!(Ok("test"), value.get_string());
    }

    #[test]
    fn get_func_on_string() {
        let value = Value::String("not a function".to_string());
        assert_eq!(
            Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::Fn {
                        args: vec![],
                        variadic_arg: Some(Type::Value.into()),
                        returns: Type::Value.into()
                    },
                    actual: Type::String
                }
                .into(),
                0..0
            )]),
            value.get_func()
        );
    }

    #[test]
    fn get_func_on_func() {
        let expected_fn: Box<BuiltinFn> = BuiltinFn {
            name: "name",
            args: &[],
            return_type: Type::Unknown,
            func: example_builtin,
        }
        .into();

        let value = Value::Fn(expected_fn.clone());

        assert_eq!(Ok(expected_fn), value.get_func());
    }

    #[test]
    fn get_func_on_bool() {
        let value = Value::Bool(true);
        assert_eq!(
            Err(vec![(
                RuntimeError::TypeMismatch {
                    expected: Type::Fn {
                        args: vec![],
                        variadic_arg: Some(Type::Value.into()),
                        returns: Type::Value.into()
                    },
                    actual: Type::Bool
                }
                .into(),
                0..0
            )]),
            value.get_func()
        );
    }

    // #[test]
    // fn get_bool_on_func() {
    //     let dummy_fn: Rc<BuiltinFn> = Rc::new(|_, _| Ok(Value::Bool(true)));
    //     assert_eq!(
    //         Err(vec![(
    //             RuntimeError::TypeMismatch {
    //                 expected: Type::Bool,
    //                 actual: Type::Fn
    //             }
    //             .into(),
    //             0..0
    //         )]),
    //         Value::Fn(dummy_fn).get_bool()
    //     );
    // }
}
