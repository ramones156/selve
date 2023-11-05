use std::collections::HashMap;

use crate::environment::Environment;

type FunctionCall = fn(args: Vec<RuntimeValue>, env: &mut Environment) -> RuntimeValue;

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeValue {
    Null,
    Object(HashMap<String, RuntimeValue>),
    Boolean(bool),
    NativeFn(FunctionCall),
    Number(String),
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Null => {
                writeln!(f, "null");
            }
            RuntimeValue::Object(map) => {
                map.iter().for_each(|(k, v)| {
                    writeln!(f, "{} -> {}", k, v);
                });
            }
            RuntimeValue::Boolean(b) => {
                writeln!(f, "{}", if *b { "true" } else { "false" });
            }
            RuntimeValue::NativeFn(call) => {
                writeln!(f, "FnCall");
            }
            RuntimeValue::Number(n) => {
                writeln!(f, "{n}");
            }
        }
        Ok(())
    }
}
