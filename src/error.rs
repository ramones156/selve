use thiserror::Error;

use crate::{ast::Stmt, token::TokenType};

pub type Result<T> = anyhow::Result<T>;

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
    #[error("Invalid assignment")]
    InvalidAssignment,
    #[error("Unsupported binary operator {0}")]
    InvalidOperator(String),
    #[error("Value {0:?} is not a function")]
    ValueNotAFunction(Stmt),
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("\nExpected {0:?} but got {1:?}.\n{2}")]
    ExpectedCharacter(TokenType, TokenType, String),
    #[error("Expected a token but it returned nothing")]
    ExpectedToken,
    #[error("Expected parameter type {0:?} to be of type string")]
    ExpectedParameterToBeString(Stmt),
    #[error("Unsupported token {0:?}")]
    UnsupportedTokenType(TokenType),
    #[error("Cannot use dot operator without rhs being an identifier")]
    NoDotOperatorWithoutRhsIdentifier,
    #[error("A value is required for const assignment")]
    ConstValueRequired,
}

#[derive(Error, Debug, PartialEq)]
pub enum LexerError {
    #[error("Unexpected character {0}")]
    UnexpectedCharacter(char),
}

#[derive(Error, Debug, PartialEq)]
pub enum InterpreterError {
    #[error("Unexpected character {0:?}")]
    UnexpectedStatement(Stmt),
}
