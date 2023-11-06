use thiserror::Error;

use crate::{ast::Stmt, token::TokenType};

#[derive(Error, Debug, PartialEq)]
pub enum EnvError {
    #[error("Cannot redeclare variable {0}")]
    RedeclareVariable(String),
    #[error("Cannot reassign to constant {0}")]
    ReassignVariable(String),
    #[error("Cannot resolve {0} since it doesnt exist")]
    VariableNotFound(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    #[error("Env error: {0:?}")]
    EnvError(EnvError),
    #[error("Invalid assignment")]
    InvalidAssignment,
    #[error("Unsupported binary operator {0}")]
    InvalidOperator(String),
    #[error("Value {0:?} is not a function")]
    ValueNotAFunction(Stmt),
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Expected {0} in statement")]
    ExpectedCharacter(String),
    #[error("Expected a token but it returned nothing")]
    ExpectedToken,
    #[error("Unsupported token {0:?}")]
    UnsupportedTokenType(TokenType),
    #[error("Cannot use dot operator without rhs being an identifier")]
    NoDotOperatorWithoutRhsIdentifier,
}
