//! The core value type used in the virtual machine.

use std::{fmt::Display, rc::Rc};

use crate::{builtins::BuiltinFn, types::Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Fn(Rc<BuiltinFn>),
    Bool(bool),
}

impl Value {
    pub fn get_type(&self) -> Type {
        self.clone().into()
    }

    pub fn get_string(&self) -> &str {
        match self {
            Value::String(s) => s.as_str(),
            _ => panic!("Value is not a string"),
        }
    }

    pub fn get_func(&self) -> Rc<BuiltinFn> {
        match self {
            Value::Fn(f) => f.clone(),
            _ => panic!("Value is not a function"),
        }
    }

    pub fn get_bool(&self) -> bool {
        match self {
            Value::Bool(s) => *s,
            _ => panic!("Value is not a string"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "`{}`", string),
            Value::Fn(builtin) => write!(f, "{builtin:?}"),
            Value::Bool(value) => write!(f, "{}", value),
        }
    }
}
