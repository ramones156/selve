use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::anyhow;

use crate::ast::*;
use crate::error::ParseError;
use crate::error::Result;
use crate::lexer::*;
use crate::token::*;

#[derive(Debug)]
pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: vec![].into_iter().peekable(),
        }
    }

    pub fn produce_ast(&mut self, src: String) -> Result<Program> {
        self.tokens = Lexer::tokenize(src)?.into_iter().peekable();

        let mut program = Program { body: vec![] };

        while let Some(t) = self.peek() {
            if t.token_type == TokenType::Eof {
                break;
            }

            program.body.push(self.parse_stmt()?);
        }

        Ok(program)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        if let Some(t) = self.peek() {
            let stmt = match t.token_type {
                TokenType::Comment => self.parse_comment_declaration(),
                TokenType::LetKeyword | TokenType::ConstKeyword => {
                    self.parse_variable_declaration()
                }
                TokenType::FnKeyword => self.parse_function_declaration(),
                _ => self.parse_expr(),
            };
            self.expect(TokenType::Semicolon, "Expected semicolon after statement");
            return stmt;
        }
        Err(anyhow!(ParseError::ExpectedToken))
    }

    fn parse_comment_declaration(&mut self) -> Result<Stmt> {
        if let Some(t) = self.peek() {
            if let TokenType::Comment = t.token_type {
                let comment = Stmt::Comment(t.value.to_owned());
                return Ok(comment);
            }
        }

        Err(anyhow!(ParseError::ExpectedToken))
    }

    fn parse_expr(&mut self) -> Result<Stmt> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Stmt> {
        let left = self.parse_object_expr()?;
        if let Some(t) = self.peek() {
            if t.token_type == TokenType::Equals {
                self.eat();
                let value = self.parse_assignment_expr()?;
                return Ok(Stmt::AssignmentExpr {
                    assignee: Box::new(left),
                    value: Box::new(value),
                });
            }
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Stmt> {
        let mut left = self.parse_multiplicative_expr()?;

        while let Some(t) = self.peek() {
            if t.value != "+" && t.value != "-" {
                break;
            }

            if let Some(tr) = self.eat() {
                let operator = tr.value;
                let right = self.parse_multiplicative_expr()?;
                left = Stmt::BinaryExpr {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                };
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Stmt> {
        let mut left = self.parse_call_member_expr()?;

        while let Some(t) = self.peek() {
            if t.value != "*" && t.value != "/" && t.value != "%" {
                break;
            }

            if let Some(tr) = self.eat() {
                let operator = tr.value;
                let right = self.parse_call_member_expr()?;
                left = Stmt::BinaryExpr {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                };
            }
        }

        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Stmt> {
        if let Some(t) = self.eat() {
            match t.token_type {
                TokenType::Identifier => Ok(Stmt::Identifier(t.value.to_owned())),
                TokenType::Number => Ok(Stmt::NumericLiteral(t.value.to_owned())),
                TokenType::LeftParen => {
                    let value = self.parse_expr()?;
                    self.expect(TokenType::RightParen, "No right paren inside expression");

                    Ok(value)
                }
                _ => Err(anyhow!(ParseError::UnsupportedTokenType(t.token_type))),
            }
        } else {
            Err(anyhow!(ParseError::ExpectedToken))
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Stmt> {
        if let Some(t) = self.eat() {
            let constant = t.token_type == TokenType::ConstKeyword;
            let identifier = self
                .expect(
                    TokenType::Identifier,
                    "Expected identifier name after let or const keyword",
                )?
                .value;
            if let Some(t) = self.peek() {
                if t.token_type == TokenType::Semicolon {
                    self.eat();
                    if constant {
                        return Err(anyhow!(ParseError::ConstValueRequired));
                    }

                    return Ok(Stmt::VarDeclaration {
                        constant,
                        identifier,
                        value: None,
                    });
                }

                self.expect(TokenType::Equals, "Expected equals token after identifier");
                let declaration = Stmt::VarDeclaration {
                    constant,
                    identifier,
                    value: Some(Box::new(self.parse_expr()?)),
                };

                return Ok(declaration);
            }
        }

        Err(anyhow!(ParseError::ExpectedToken))
    }

    fn parse_function_declaration(&mut self) -> Result<Stmt> {
        self.eat();
        let name = self
            .expect(
                TokenType::Identifier,
                "Expected function name following fn keyword",
            )?
            .value;

        let args = self.parse_args()?;

        let mut parameters = vec![];
        for arg in args {
            if let Stmt::Identifier(v) = arg {
                parameters.push(v);
            } else {
                return Err(anyhow!(ParseError::ExpectedParameterToBeString(arg)));
            }
        }

        self.expect(
            TokenType::LeftBrace,
            "Expected function body following declaration",
        );

        let mut body = vec![];
        while let Some(t) = self.peek() {
            if t.token_type == TokenType::Eof || t.token_type == TokenType::RightBrace {
                break;
            }

            body.push(self.parse_stmt()?);
        }

        self.expect(TokenType::RightBrace, "Closing bracket expected");

        let function = Stmt::FnDeclaration {
            name,
            parameters,
            body,
            is_const: false,
        };
        Ok(function)
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn eat(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn expect(&mut self, token_type: TokenType, err: &str) -> Result<Token> {
        if let Some(t) = self.eat() {
            if t.token_type != token_type {
                return Err(anyhow!(ParseError::ExpectedCharacter(
                    token_type,
                    t.token_type,
                    err.to_owned()
                )));
            }
            Ok(t)
        } else {
            Err(anyhow!(ParseError::ExpectedToken))
        }
    }

    /// { foo: foo, bar, baz: null }
    fn parse_object_expr(&mut self) -> Result<Stmt> {
        if let Some(t) = self.peek() {
            if t.token_type != TokenType::LeftBrace {
                return self.parse_additive_expr();
            }

            self.eat();
            let mut properties = vec![];

            while let Some(t) = self.peek() {
                if t.token_type == TokenType::RightBrace || t.token_type == TokenType::Eof {
                    break;
                }

                let key = self
                    .expect(TokenType::Identifier, "Object literal identifier expected")?
                    .value;

                if let Some(t) = self.peek() {
                    if t.token_type == TokenType::Comma {
                        // pair -> { key }
                        self.eat();
                        properties.push(Property { key, value: None });
                        continue;
                    } else if t.token_type == TokenType::RightBrace {
                        // pair -> { key, }
                        properties.push(Property { key, value: None });
                        continue;
                    }

                    self.expect(
                        TokenType::Colon,
                        "Missing colon after identifier in object expression",
                    );

                    let value = self.parse_expr()?;
                    properties.push(Property {
                        key,
                        value: Some(Box::new(value)),
                    });

                    if let Some(t) = self.peek() {
                        if t.token_type != TokenType::RightBrace {
                            self.expect(
                                TokenType::Comma,
                                "Expected comma or closing bracket after property",
                            );
                        }
                    }
                }
            }
            self.expect(
                TokenType::RightBrace,
                "Object literal is missing a closing brace",
            );

            return Ok(Stmt::ObjectLiteral(properties));
        }

        Err(anyhow!(ParseError::ExpectedToken))
    }

    fn parse_call_member_expr(&mut self) -> Result<Stmt> {
        let member = self.parse_member_expr()?;

        if let Some(t) = self.peek() {
            if t.token_type == TokenType::LeftParen {
                return self.parse_call_expr(member);
            }
        }

        Ok(member)
    }

    /// foo(...args)
    /// ^..........^
    fn parse_call_expr(&mut self, caller: Stmt) -> Result<Stmt> {
        let mut call_expr = Stmt::CallExpr {
            caller: Box::new(caller),
            args: self.parse_args()?,
        };

        if let Some(t) = self.peek() {
            if t.token_type == TokenType::LeftParen {
                call_expr = self.parse_call_expr(call_expr)?;
            }
        }
        Ok(call_expr)
    }

    /// foo(...args)
    ///     ^.....^
    fn parse_args(&mut self) -> Result<Vec<Stmt>> {
        self.expect(TokenType::LeftParen, "Expected open parenthesis");
        if let Some(t) = self.peek() {
            let args = if t.token_type == TokenType::RightParen {
                vec![] // args list is empty
            } else {
                self.parse_args_list()?
            };

            // TODO for nested function parse args takes both parens
            self.expect(
                TokenType::RightParen,
                "Missing closing parenthesis in argument list",
            )?;

            return Ok(args);
        };

        Err(anyhow!(ParseError::ExpectedToken))
    }

    fn parse_args_list(&mut self) -> Result<Vec<Stmt>> {
        let mut args = vec![self.parse_assignment_expr()?];

        while let Some(t) = self.peek() {
            if t.token_type != TokenType::Comma || self.eat().is_none() {
                break;
            }

            args.push(self.parse_assignment_expr()?);
        }

        Ok(args)
    }

    fn parse_member_expr(&mut self) -> Result<Stmt> {
        let mut object = self.parse_primary_expr()?;

        while let Some(t) = self.peek() {
            if t.token_type != TokenType::Dot && t.token_type != TokenType::LeftBracket {
                break;
            }

            if let Some(operator) = self.eat() {
                let computed;
                let property;

                if operator.token_type == TokenType::Dot {
                    property = self.parse_primary_expr()?;

                    if let Stmt::Identifier(_) = property {
                        continue;
                    } else {
                        return Err(anyhow!(ParseError::NoDotOperatorWithoutRhsIdentifier));
                    }
                } else {
                    computed = true;
                    property = self.parse_expr()?;
                    self.expect(
                        TokenType::RightBracket,
                        "Missing closing bracket in computed member expression",
                    )?;
                }

                object = Stmt::MemberExpr {
                    object: Box::new(object),
                    property: Box::new(property),
                    computed,
                };
            }
        }

        Ok(object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Stmt::{AssignmentExpr, FnDeclaration};

    #[test]
    fn basic() {
        let expected = Program {
            body: vec![Stmt::BinaryExpr {
                left: Box::new(Stmt::NumericLiteral("45".to_owned())),
                right: Box::new(Stmt::BinaryExpr {
                    left: Box::new(Stmt::BinaryExpr {
                        left: Box::new(Stmt::Identifier("foo".to_owned())),
                        right: Box::new(Stmt::NumericLiteral("4".to_owned())),
                        operator: "+".to_owned(),
                    }),
                    right: Box::new(Stmt::Identifier("bar".to_owned())),
                    operator: "%".to_owned(),
                }),
                operator: "+".to_owned(),
            }],
        };
        let input = "45 + (foo + 4) % bar";
        let mut parser = Parser::new();

        let program = parser
            .produce_ast(input.to_owned())
            .expect("Unable to parse");
        assert_eq!(program, expected);
    }

    #[test]
    fn assignment() {
        let expected = Program {
            body: vec![
                Stmt::VarDeclaration {
                    constant: false,
                    identifier: "foo".to_string(),
                    value: Some(Box::new(Stmt::BinaryExpr {
                        left: Box::new(Stmt::NumericLiteral("50".to_string())),
                        right: Box::new(Stmt::NumericLiteral("2".to_string())),
                        operator: "/".to_string(),
                    })),
                },
                Stmt::VarDeclaration {
                    constant: true,
                    identifier: "bar".to_string(),
                    value: Some(Box::new(Stmt::ObjectLiteral(vec![
                        Property {
                            key: "x".to_string(),
                            value: Some(Box::new(Stmt::NumericLiteral("100".to_string()))),
                        },
                        Property {
                            key: "y".to_string(),
                            value: Some(Box::new(Stmt::NumericLiteral("32".to_string()))),
                        },
                        Property {
                            key: "foo".to_string(),
                            value: None,
                        },
                        Property {
                            key: "baz".to_string(),
                            value: Some(Box::new(Stmt::ObjectLiteral(vec![Property {
                                key: "z".to_string(),
                                value: Some(Box::new(Stmt::Identifier("true".to_string()))),
                            }]))),
                        },
                    ]))),
                },
            ],
        };

        let input = r#"
            let foo = 50 / 2;
            const bar = {
                x: 100,
                y: 32,
                foo,
                baz: {
                    z: true,
                },
            };
        "#;

        let mut parser = Parser::new();

        let program = parser
            .produce_ast(input.to_string())
            .expect("Unable to parse");
        assert_eq!(program, expected);
    }

    #[test]
    fn declare_function() {
        let expected = Program {
            body: vec![FnDeclaration {
                name: "add".to_owned(),
                parameters: vec!["x".to_owned(), "y".to_owned()],
                body: vec![
                    Stmt::FnDeclaration {
                        name: "subtract".to_owned(),
                        parameters: vec![],
                        body: vec![Stmt::CallExpr {
                            args: vec![],
                            caller: Box::new(Stmt::Identifier("print".to_owned())),
                        }],
                        is_const: false,
                    },
                    AssignmentExpr {
                        assignee: Box::new(Stmt::Identifier("result".to_owned())),
                        value: Box::new(Stmt::BinaryExpr {
                            left: Box::new(Stmt::Identifier("x".to_owned())),
                            right: Box::new(Stmt::Identifier("y".to_owned())),
                            operator: "+".to_owned(),
                        }),
                    },
                    Stmt::CallExpr {
                        args: vec![Stmt::Identifier("result".to_owned())],
                        caller: Box::new(Stmt::Identifier("print".to_owned())),
                    },
                    Stmt::Identifier("result".to_owned()),
                ],
                is_const: false,
            }],
        };

        let input = r#"
            fn add(x,y) {
                fn subtract() {
                    print();
                }

                let result = x + y;
                
                print(result);
                result
            }
        "#;

        let mut parser = Parser::new();

        let program = parser
            .produce_ast(input.to_string())
            .expect("Unable to parse");
        assert_eq!(program, expected);
    }

    #[test]
    fn comment() {
        let expected = Program {
            body: vec![
                Stmt::Comment(" this is a comment!".to_owned()),
                Stmt::VarDeclaration {
                    constant: false,
                    identifier: "foo".to_owned(),
                    value: Some(Box::new(Stmt::BinaryExpr {
                        left: Box::new(Stmt::NumericLiteral("50".to_owned())),
                        right: Box::new(Stmt::NumericLiteral("2".to_owned())),
                        operator: "/".to_owned(),
                    })),
                },
                Stmt::Comment(" this does stuff".to_owned()),
                Stmt::CallExpr {
                    args: vec![Stmt::BinaryExpr {
                        left: Box::new(Stmt::BinaryExpr {
                            left: Box::new(Stmt::NumericLiteral("40".to_owned())),
                            right: Box::new(Stmt::NumericLiteral("2".to_owned())),
                            operator: "*".to_owned(),
                        }),
                        right: Box::new(Stmt::Identifier("foo".to_owned())),
                        operator: "+".to_owned(),
                    }],
                    caller: Box::new(Stmt::Identifier("print".to_owned())),
                },
                Stmt::Comment(" so does this!".to_owned()),
            ],
        };

        let input = r#"
            // this is a comment!
            let foo = 50 / 2;

            // this does stuff
            print(40 * 2 + foo); // so does this!
        "#;

        let mut parser = Parser::new();

        let program = parser
            .produce_ast(input.to_string())
            .expect("Unable to parse");
        assert_eq!(program, expected);
    }

    #[test]
    fn call_expression() {
        let expected = Program {
            body: vec![
                Stmt::VarDeclaration {
                    constant: false,
                    identifier: "foo".to_string(),
                    value: Some(Box::new(Stmt::BinaryExpr {
                        left: Box::new(Stmt::NumericLiteral("50".to_string())),
                        right: Box::new(Stmt::NumericLiteral("2".to_string())),
                        operator: "/".to_string(),
                    })),
                },
                Stmt::CallExpr {
                    args: vec![Stmt::BinaryExpr {
                        left: Box::new(Stmt::BinaryExpr {
                            left: Box::new(Stmt::NumericLiteral("40".to_string())),
                            right: Box::new(Stmt::NumericLiteral("2".to_string())),
                            operator: "*".to_string(),
                        }),
                        right: Box::new(Stmt::Identifier("foo".to_string())),
                        operator: "+".to_string(),
                    }],
                    caller: Box::new(Stmt::Identifier("print".to_string())),
                },
            ],
        };

        let input = r#"
            let foo = 50 / 2;

            print(40 * 2 + foo);
        "#;

        let mut parser = Parser::new();

        let program = parser
            .produce_ast(input.to_string())
            .expect("Unable to parse");
        assert_eq!(program, expected);
    }

    #[test]
    fn nested_call_expression() {
        let expected = Program {
            body: vec![Stmt::CallExpr {
                args: vec![Stmt::CallExpr {
                    args: vec![Stmt::NumericLiteral("5".to_owned())],
                    caller: Box::new(Stmt::Identifier("print".to_owned())),
                }],
                caller: Box::new(Stmt::Identifier("print".to_string())),
            }],
        };

        let input = r#"print(print(5));"#;

        let mut parser = Parser::new();

        let program = parser
            .produce_ast(input.to_string())
            .expect("Unable to parse");
        assert_eq!(program, expected);
    }

    #[test]
    fn const_requires_value() {
        let input1 = r#"const foo;"#;
        let input2 = r#"const foo = 6;"#;

        let mut parser = Parser::new();

        let error = parser.produce_ast(input1.to_string()).unwrap_err();
        let result = parser.produce_ast(input2.to_string());

        assert_eq!(
            "A value is required for const assignment",
            error.to_string()
        );
        assert!(result.is_ok());
    }
}
