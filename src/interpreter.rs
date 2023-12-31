use std::collections::HashMap;

use anyhow::anyhow;

use crate::{
    ast::{Property, Stmt},
    environment::Environment,
    error::{EvalError, InterpreterError, Result},
    values::RuntimeValue,
};

pub fn evaluate(stmt: Stmt, env: &mut Environment) -> Result<RuntimeValue> {
    match stmt {
        Stmt::NumericLiteral(v) => Ok(RuntimeValue::Number(v)),
        Stmt::Identifier(v) => eval_identifier(v, env),
        Stmt::ObjectLiteral(properties) => eval_object_expr(properties, env),
        Stmt::CallExpr { args, caller } => eval_call_expr(args, *caller, env),
        Stmt::AssignmentExpr { assignee, value } => eval_assignment_expr(*assignee, *value, env),
        Stmt::FnDeclaration {
            name,
            parameters,
            body,
            is_const,
        } => eval_function_declaration(name, parameters, body, is_const, env),
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
        _ => Err(anyhow!(InterpreterError::UnexpectedStatement(stmt))),
    }
}

fn eval_call_expr(args: Vec<Stmt>, caller: Stmt, env: &mut Environment) -> Result<RuntimeValue> {
    let args = args
        .iter()
        .map(|arg| evaluate(arg.to_owned(), env).expect("Cannot evaludate argument {arg:?}"))
        .collect::<Vec<_>>();

    let call_expr = evaluate(caller.to_owned(), env)?;

    match call_expr {
        RuntimeValue::NativeFn(function) => {
            let result = function(args, env);
            Ok(result)
        }
        RuntimeValue::Function {
            name: _,
            parameters,
            declaration_env,
            body,
        } => {
            let mut scope = Environment::with(declaration_env);

            for (i, parameter) in parameters.iter().enumerate() {
                scope.declare_var(parameter, args[i].clone(), false)?;
            }

            let mut result = RuntimeValue::Null;

            for stmt in body {
                result = evaluate(stmt, &mut scope)?;
            }

            Ok(result)
        }
        _ => Err(anyhow!(EvalError::ValueNotAFunction(caller))),
    }
}

fn eval_program(program: crate::ast::Program, env: &mut Environment) -> Result<RuntimeValue> {
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
) -> Result<RuntimeValue> {
    let mut properties = HashMap::new();
    for property in object_properties {
        let Property { key, value } = property;

        let value = if let Some(value) = value {
            evaluate(*value, env)?
        } else {
            match env.lookup_var(&key) {
                Ok(v) => v,
                Err(e) => {
                    return Err(anyhow!(e));
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
) -> Result<RuntimeValue> {
    if let Stmt::Identifier(name) = assignee {
        let value = evaluate(value, env)?;

        return match env.assign_var(&name, value) {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow!(e)),
        };
    }

    Err(anyhow!(EvalError::InvalidAssignment))
}

fn eval_function_declaration(
    name: String,
    parameters: Vec<String>,
    body: Vec<Stmt>,
    _is_const: bool,
    env: &mut Environment,
) -> Result<RuntimeValue> {
    let function = RuntimeValue::Function {
        name: name.clone(),
        parameters,
        declaration_env: env.clone(),
        body,
    };

    env.declare_var(&name, function, true)
}

fn eval_variable_declaration(
    constant: bool,
    identifier: String,
    value: Option<Box<Stmt>>,
    env: &mut Environment,
) -> Result<RuntimeValue> {
    let runtime_value = if let Some(value) = value {
        evaluate(*value, env)?
    } else {
        RuntimeValue::Null
    };

    match env.declare_var(&identifier, runtime_value, constant) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow!(e)),
    }
}

fn eval_identifier(v: String, env: &mut Environment) -> Result<RuntimeValue> {
    match env.lookup_var(&v) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow!(e)),
    }
}

fn evaluate_binary_expr(
    left: Stmt,
    right: Stmt,
    operator: String,
    env: &mut Environment,
) -> Result<RuntimeValue> {
    if let RuntimeValue::Number(lhs) = evaluate(left, env)? {
        if let RuntimeValue::Number(rhs) = evaluate(right, env)? {
            return eval_numeric_binary_expr(lhs, rhs, operator);
        }
    }

    Ok(RuntimeValue::Null)
}

fn eval_numeric_binary_expr(lhs: String, rhs: String, operator: String) -> Result<RuntimeValue> {
    let lhs = lhs.parse::<i64>().unwrap();
    let rhs = rhs.parse::<i64>().unwrap();

    let result = match &*operator {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        "%" => lhs % rhs,
        _ => {
            return Err(anyhow!(EvalError::InvalidOperator(operator)));
        }
    };

    Ok(RuntimeValue::Number(result.to_string()))
}
