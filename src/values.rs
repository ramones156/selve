use std::collections::HashMap;



#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Null,
    Object(HashMap<String, RuntimeValue>),
    Boolean(bool),
    Number(String),
}
