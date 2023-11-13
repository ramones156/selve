use anyhow::anyhow;
use std::{
    collections::{HashMap, HashSet},
};

use crate::{
    error::{EnvError, Result},
    values::{self, RuntimeValue},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    parent: Option<Box<Environment>>,
    variables: HashMap<String, RuntimeValue>,
    constants: HashSet<String>,
}

impl Environment {
    pub fn new() -> Self {
        let mut global = Self {
            parent: None,
            variables: HashMap::new(),
            constants: HashSet::new(),
        };

        global.setup_scope();

        global
    }

    fn setup_scope(&mut self) {
        self.declare_var("true", RuntimeValue::Boolean(true), true);
        self.declare_var("false", RuntimeValue::Boolean(false), true);
        self.declare_var("null", RuntimeValue::Null, true);

        fn print(
            args: Vec<values::RuntimeValue>,
            _environment: &mut Environment,
        ) -> values::RuntimeValue {
            args.iter().for_each(|arg| println!("{arg}"));

            RuntimeValue::Null
        }
        self.declare_var("print", RuntimeValue::NativeFn(print), true);
        fn time(
            _args: Vec<values::RuntimeValue>,
            _environment: &mut Environment,
        ) -> values::RuntimeValue {
            RuntimeValue::Number("Friday 13th".to_string())
        }
        self.declare_var("time", RuntimeValue::NativeFn(time), true);
    }

    pub fn with(parent_env: Environment) -> Self {
        Self {
            parent: Some(Box::new(parent_env)),
            variables: HashMap::new(),
            constants: HashSet::new(),
        }
    }

    pub fn declare_var(
        &mut self,
        name: &str,
        value: RuntimeValue,
        constant: bool,
    ) -> Result<RuntimeValue> {
        if self.variables.contains_key(name) {
            return Err(anyhow!(EnvError::RedeclareVariable(name.to_string())));
        }

        if constant {
            self.constants.insert(name.to_owned());
        }

        self.variables.insert(name.to_string(), value.clone());
        Ok(value)
    }

    pub fn assign_var(&mut self, name: &str, value: RuntimeValue) -> Result<RuntimeValue> {
        let env = self.resolve(name)?;

        if env.constants.contains(name) {
            return Err(anyhow!(EnvError::ReassignVariable(name.to_string())));
        }

        env.variables.insert(name.to_owned(), value.clone());
        Ok(value)
    }

    pub fn lookup_var(&mut self, name: &str) -> Result<RuntimeValue> {
        let env = self.resolve(name)?;
        let value = env
            .variables
            .get(name)
            .expect("Variable was resolved but doesnt exist");
        Ok(value.clone())
    }

    pub fn resolve(&mut self, name: &str) -> Result<&mut Environment> {
        if self.variables.contains_key(name) {
            return Ok(self);
        }

        if self.parent.is_none() {
            return Err(anyhow!(EnvError::VariableNotFound(name.to_string())));
        }

        self.parent.as_mut().unwrap().resolve(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut env = Environment::new();
        assert_eq!(RuntimeValue::Boolean(true), env.lookup_var("true").unwrap());
        assert_eq!(
            RuntimeValue::Boolean(false),
            env.lookup_var("false").unwrap()
        );
        assert_eq!(RuntimeValue::Null, env.lookup_var("null").unwrap());
        assert_eq!(
            "Cannot resolve foo since it doesnt exist",
            env.lookup_var("foo")
                .expect_err("Should not be Ok()")
                .to_string(),
        );
    }
}
