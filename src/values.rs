use std::collections::HashMap;

use crate::ast::Property;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Null,
    Object(HashMap<String, RuntimeValue>),
    Boolean(bool),
    Number(String),
}
