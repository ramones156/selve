use std::collections::HashMap;

use crate::{ast::Stmt, environment::Environment};

type FunctionCall = fn(args: Vec<RuntimeValue>, env: &mut Environment) -> RuntimeValue;

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeValue {
    Null,
    Object(HashMap<String, RuntimeValue>),
    Boolean(bool),
    Function {
        name: String,
        parameters: Vec<String>,
        declaration_env: Environment,
        body: Vec<Stmt>,
    },
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
            RuntimeValue::Function {
                name,
                parameters: _,
                declaration_env: _,
                body: _,
            } => {
                writeln!(f, "{}()", name);
            }
            RuntimeValue::NativeFn(_call) => {
                writeln!(f, "FnCall");
            }
            RuntimeValue::Number(n) => {
                writeln!(f, "{n}");
            }
        }
        Ok(())
    }
}
