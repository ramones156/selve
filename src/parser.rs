use std::iter::Peekable;
use std::vec::IntoIter;

use crate::ast::*;
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

    pub fn produce_ast(&mut self, src: String) -> Program {
        self.tokens = Lexer::tokenize(src).into_iter().peekable();

        let mut program = Program { body: vec![] };

        while let Some(t) = self.peek() {
            if t.token_type == TokenType::Eof {
                break;
            }

            program.body.push(self.parse_stmt());
        }

        program
    }

    fn parse_stmt(&mut self) -> Stmt {
        if let Some(t) = self.peek() {
            return match (t.token_type) {
                TokenType::LetKeyword | TokenType::ConstKeyword => {
                    self.parse_variable_declaration()
                }
                _ => self.parse_expr(),
            };
        }
        panic!("Statement not completed");
    }

    fn parse_expr(&mut self) -> Stmt {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Stmt {
        let left = self.parse_object_expr();
        if let Some(t) = self.peek() {
            if t.token_type == TokenType::Equals {
                self.eat();
                let value = self.parse_assignment_expr();
                return Stmt::AssignmentExpr {
                    assignee: Box::new(left),
                    value: Box::new(value),
                };
            }
        }

        left
    }

    fn parse_additive_expr(&mut self) -> Stmt {
        let mut left = self.parse_multiplicative_expr();

        while let Some(t) = self.peek() {
            if (t.value != "+" && t.value != "-") {
                break;
            }

            if let Some(tr) = self.eat() {
                let operator = tr.value;
                let right = self.parse_multiplicative_expr();
                left = Stmt::BinaryExpr {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                };
            }
        }

        left
    }

    fn parse_multiplicative_expr(&mut self) -> Stmt {
        let mut left = self.parse_call_member_expr();

        while let Some(t) = self.peek() {
            if (t.value != "*" && t.value != "/" && t.value != "%") {
                break;
            }

            if let Some(tr) = self.eat() {
                let operator = tr.value;
                let right = self.parse_call_member_expr();
                left = Stmt::BinaryExpr {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                };
            }
        }

        left
    }

    fn parse_primary_expr(&mut self) -> Stmt {
        if let Some(t) = self.eat() {
            match t.token_type {
                TokenType::Identifier => Stmt::Identifier(t.value.to_owned()),
                TokenType::Number => Stmt::NumericLiteral(t.value.to_owned()),
                TokenType::LeftParen => {
                    let value = self.parse_expr();
                    self.expect(TokenType::RightParen, "No right paren inside expression");

                    value
                }
                _ => panic!("Unsupported token {t:?}"),
            }
        } else {
            panic!("No token found inside expr");
        }
    }

    fn parse_variable_declaration(&mut self) -> Stmt {
        if let Some(t) = self.eat() {
            let constant = t.token_type == TokenType::ConstKeyword;
            let identifier = self
                .expect(
                    TokenType::Identifier,
                    "Expected identifier name after let or const keyword",
                )
                .value;
            if let Some(t) = self.peek() {
                if (t.token_type == TokenType::Semicolon) {
                    self.eat();
                    if (constant) {
                        panic!("A value is required for const assignment");
                    }

                    return Stmt::VarDeclaration {
                        constant,
                        identifier,
                        value: None,
                    };
                }

                self.expect(TokenType::Equals, "Expected equals token after identifier");
                let declaration = Stmt::VarDeclaration {
                    constant,
                    identifier,
                    value: Some(Box::new(self.parse_expr())),
                };

                self.expect(TokenType::Semicolon, "Expected semicolon after expression");

                return declaration;
            }
        }

        panic!("Cannot parse variable declaration");
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn eat(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn expect(&mut self, token_type: TokenType, err: &str) -> Token {
        if let Some(t) = self.eat() {
            if (t.token_type != token_type) {
                panic!("{err}");
            }
            t
        } else {
            panic!("No token found after consuming");
        }
    }

    /// { foo: foo, bar, baz: null }
    fn parse_object_expr(&mut self) -> Stmt {
        if let Some(t) = self.peek() {
            if (t.token_type != TokenType::LeftBrace) {
                return self.parse_additive_expr();
            }

            self.eat();
            let mut properties = vec![];

            while let Some(t) = self.peek() {
                if (t.token_type == TokenType::RightBrace || t.token_type == TokenType::Eof) {
                    break;
                }

                let key = self
                    .expect(TokenType::Identifier, "Object literal identifier expected")
                    .value;

                if let Some(t) = self.peek() {
                    if (t.token_type == TokenType::Comma) {
                        // pair -> { key }
                        self.eat();
                        properties.push(Property { key, value: None });
                        continue;
                    } else if (t.token_type == TokenType::RightBrace) {
                        // pair -> { key, }
                        properties.push(Property { key, value: None });
                        continue;
                    }

                    self.expect(
                        TokenType::Colon,
                        "Missing colon after identifier in object expression",
                    );

                    let value = self.parse_expr();
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
            return Stmt::ObjectLiteral(properties);
        }

        panic!("Object literal cannot be parsed");
    }

    fn parse_call_member_expr(&mut self) -> Stmt {
        let member = self.parse_member_expr();

        if let Some(t) = self.peek() {
            if t.token_type == TokenType::LeftParen {
                return self.parse_call_expr(member);
            }
        }

        member
    }

    fn parse_call_expr(&mut self, caller: Stmt) -> Stmt {
        let mut call_expr = Stmt::CallExpr {
            caller: Box::new(caller),
            args: self.parse_args(),
        };

        if let Some(t) = self.peek() {
            if t.token_type == TokenType::LeftParen {
                call_expr = self.parse_call_expr(call_expr);
            }
        }

        call_expr
    }

    fn parse_args(&mut self) -> Vec<Stmt> {
        self.expect(TokenType::LeftParen, "Expected open parenthesis");
        if let Some(t) = self.peek() {
            let args = if t.token_type == TokenType::RightParen {
                vec![]
            } else {
                self.parse_args_list()
            };
            self.expect(TokenType::RightParen, "Missing closing parenthesis");
            return args;
        };

        panic!("No token after argument tokens");
    }

    fn parse_args_list(&mut self) -> Vec<Stmt> {
        let mut args = vec![self.parse_expr()];

        while let Some(t) = self.peek() {
            if (t.token_type != TokenType::Comma || self.eat().is_none()) {
                break;
            }
            args.push(self.parse_assignment_expr());
        }

        args
    }

    fn parse_member_expr(&mut self) -> Stmt {
        let mut object = self.parse_primary_expr();

        while let Some(t) = self.peek() {
            if (t.token_type != TokenType::Dot && t.token_type != TokenType::LeftBracket) {
                break;
            }

            let mut computed;
            let mut property;

            if let Some(operator) = self.peek() {
                if operator.token_type == TokenType::Dot {
                    computed = false;
                    property = self.parse_primary_expr();

                    if let Stmt::Identifier(_) = property {
                        continue;
                    } else {
                        panic!("Cannot use dot operator without rhs being an identifier");
                    }
                } else {
                    computed = true;
                    property = self.parse_expr();
                    self.expect(
                        TokenType::RightBracket,
                        "Missing closing bracket in computed member expression",
                    );
                }
                object = Stmt::MemberExpr {
                    object: Box::new(object),
                    property: Box::new(property),
                    computed,
                };
            }
        }

        object
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_example() {
        let input = "45 + (foo + 4) % bar";
        let mut parser = Parser::new();

        let program = parser.produce_ast(input.to_owned());
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
        assert_eq!(program, expected);
    }
}
