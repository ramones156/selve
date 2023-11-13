#![allow(unused)] // temp for development

use std::{
    io::{stdin, stdout, Write},
    process::exit,
};

use environment::Environment;
use parser::Parser;

use crate::interpreter::evaluate;
mod ast;
mod environment;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod values;

fn main() {
    repl();
}

fn repl() {
    let mut env = Environment::new();
    let mut parser = Parser::new();

    loop {
        let input = prompt();

        if input.is_empty() || input == "exit" {
            exit(1);
        }

        let program = parser.produce_ast(input).expect("Unable to parse");
        println!("{:?}", program);

        let result = evaluate(ast::Stmt::Program(program), &mut env);
    }
}

fn prompt() -> String {
    let mut input = String::new();
    let stdin = stdin();
    let mut stdout = stdout();

    stdout.write_all(b"> ");
    stdout.flush();
    stdin.read_line(&mut input);

    input
}
