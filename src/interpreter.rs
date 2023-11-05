use std::collections::{HashMap};

use crate::{
    ast::{Property, Stmt},
    environment::Environment,
    error::EvalError,
    values::RuntimeValue,
};

pub fn evaluate(stmt: Stmt, env: &mut Environment) -> Result<RuntimeValue, EvalError> {
    match stmt {
        Stmt::NumericLiteral(v) => Ok(RuntimeValue::Number(v)),
        Stmt::Identifier(v) => eval_identifier(v, env),
        Stmt::ObjectLiteral(properties) => eval_object_expr(properties, env),
        Stmt::AssignmentExpr { assignee, value } => eval_assignment_expr(*assignee, *value, env),
        Stmt::VarDeclaration {
            constant,
            identifier,
            value,
        } => eval_variable_declaration(constant, identifier, value, env),
        Stmt::BinaryExpr {
            left,
            right,
            operator,
        } => evaluate_binary_expr(*left, *right, operator, env),
        Stmt::Program(program) => eval_program(program, env),
        _ => panic!("AST Node has not been set up. \n{stmt:?}"),
    }
}

fn eval_program(
    program: crate::ast::Program,
    env: &mut Environment,
) -> Result<RuntimeValue, EvalError> {
    let mut last_evaluated = RuntimeValue::Null;

    for statement in program.body {
        match evaluate(statement, env) {
            Ok(v) => last_evaluated = v,
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(last_evaluated)
}

fn eval_object_expr(
    object_properties: Vec<Property>,
    env: &mut Environment,
) -> Result<RuntimeValue, EvalError> {
    let mut properties = HashMap::new();
    for property in object_properties {
        let Property { key, value } = property;

        let value = if let Some(value) = value {
            evaluate(*value, env)?
        } else {
            match env.lookup_var(&key) {
                Ok(v) => v,
                Err(e) => {
                    return Err(EvalError::EnvError(e));
                }
            }
        };

        properties.insert(key, value);
    }

    Ok(RuntimeValue::Object(properties))
}

fn eval_assignment_expr(
    assignee: Stmt,
    value: Stmt,
    env: &mut Environment,
) -> Result<RuntimeValue, EvalError> {
    if let Stmt::Identifier(name) = assignee {
        let value = evaluate(value, env)?;

        return match env.assign_var(&name, value) {
            Ok(v) => Ok(v),
            Err(e) => Err(EvalError::EnvError(e)),
        };
    }

    Err(EvalError::InvalidAssignment)
}

fn eval_variable_declaration(
    constant: bool,
    identifier: String,
    value: Option<Box<Stmt>>,
    env: &mut Environment,
) -> Result<RuntimeValue, EvalError> {
    let runtime_value = if let Some(value) = value {
        evaluate(*value, env)?
    } else {
        RuntimeValue::Null
    };

    match env.declare_var(&identifier, runtime_value, constant) {
        Ok(v) => Ok(v),
        Err(e) => Err(EvalError::EnvError(e)),
    }
}

fn eval_identifier(v: String, env: &mut Environment) -> Result<RuntimeValue, EvalError> {
    match env.lookup_var(&v) {
        Ok(v) => Ok(v),
        Err(e) => Err(EvalError::EnvError(e)),
    }
}

fn evaluate_binary_expr(
    left: Stmt,
    right: Stmt,
    operator: String,
    env: &mut Environment,
) -> Result<RuntimeValue, EvalError> {
    if let RuntimeValue::Number(lhs) = evaluate(left, env)? {
        if let RuntimeValue::Number(rhs) = evaluate(right, env)? {
            return eval_numeric_binary_expr(lhs, rhs, operator);
        }
    }

    Ok(RuntimeValue::Null)
}

fn eval_numeric_binary_expr(
    lhs: String,
    rhs: String,
    operator: String,
) -> Result<RuntimeValue, EvalError> {
    let lhs = lhs.parse::<i64>().unwrap();
    let rhs = rhs.parse::<i64>().unwrap();

    let result = match &*operator {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        "%" => lhs % rhs,
        _ => {
            return Err(EvalError::InvalidOperator(operator));
        }
    };

    Ok(RuntimeValue::Number(result.to_string()))
}
