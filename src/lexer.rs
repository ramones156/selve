use crate::{error::LexerError, token::*};

pub struct Lexer;

impl Lexer {
    pub fn tokenize(src: impl Into<String>) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];
        let src: String = src.into();

        let mut src = src.chars().peekable();

        while let Some(c) = src.next() {
            if Self::is_skippable(c) {
                continue;
            }

            let token = match c {
                '(' => Token::new(c.to_string(), TokenType::LeftParen),
                ')' => Token::new(c.to_string(), TokenType::RightParen),
                '{' => Token::new(c.to_string(), TokenType::LeftBrace),
                '[' => Token::new(c.to_string(), TokenType::LeftBracket),
                ']' => Token::new(c.to_string(), TokenType::RightBracket),
                '}' => Token::new(c.to_string(), TokenType::RightBrace),
                '/' => {
                    if let Some(c2) = src.peek() {
                        if *c2 != '/' && *c2 != '*' {
                            Token::new(c.to_string(), TokenType::BinaryOperator)
                        } else {
                            let mut comment = String::new();
                            if let Some(c2) = src.next() {
                                match c2 {
                                    '/' => {
                                        // Single-line comment
                                        for next_char in src.by_ref() {
                                            if next_char == '\n' {
                                                break;
                                            }
                                            comment.push(next_char);
                                        }
                                    }
                                    '*' => {
                                        // Multi-line comment
                                        while let Some(next_char) = src.next() {
                                            if next_char == '*' {
                                                if let Some('/') = src.next() {
                                                    break;
                                                }
                                                comment.push(next_char);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Token::new(comment, TokenType::Comment)
                        }
                    } else {
                        Token::new(c.to_string(), TokenType::BinaryOperator)
                    }
                }
                '+' | '-' | '*' | '%' => Token::new(c.to_string(), TokenType::BinaryOperator),
                '=' => Token::new(c.to_string(), TokenType::Equals),
                '.' => Token::new(c.to_string(), TokenType::Dot),
                ',' => Token::new(c.to_string(), TokenType::Comma),
                ':' => Token::new(c.to_string(), TokenType::Colon),
                ';' => Token::new(c.to_string(), TokenType::Semicolon),
                _ => {
                    if Self::is_numeric(c) {
                        let mut num = String::new();
                        num.push(c);

                        while let Some(c) = src.peek() {
                            if !Self::is_numeric(*c) {
                                break;
                            }
                            num.push(src.next().unwrap());
                        }
                        Token::new(num, TokenType::Number)
                    } else if Self::is_alpha(c) {
                        let mut ident = String::new();
                        ident.push(c);

                        while let Some(c) = src.peek() {
                            if !Self::is_ident_compliant(*c) {
                                break;
                            }
                            ident.push(src.next().unwrap());
                        }

                        if let Some(reserved_token) = TokenType::from_keyword(&ident) {
                            Token::new(ident, reserved_token)
                        } else {
                            Token::new(ident, TokenType::Identifier)
                        }
                    } else if Self::is_skippable(c) {
                        continue;
                    } else {
                        return Err(LexerError::UnexpectedCharacter(c));
                    }
                }
            };

            tokens.push(token);
        }

        tokens.push(Token::new("", TokenType::Eof));
        Ok(tokens)
    }

    fn is_ident_compliant(c: char) -> bool {
        Self::is_alpha(c) || c == '_'
    }

    fn is_alpha(c: char) -> bool {
        c.is_alphabetic()
    }

    fn is_numeric(c: char) -> bool {
        c.is_numeric()
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_alpha(c) || Self::is_numeric(c)
    }

    fn is_skippable(c: char) -> bool {
        c.is_whitespace() || c == '\n' || c == '\t' || c == '\r'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert(token: &Token, value: &str, token_type: TokenType) {
        assert_eq!(
            Token {
                value: value.to_string(),
                token_type
            },
            *token
        );
    }

    #[test]
    fn basic() {
        let src = r#"let x  = 5 + (4 / 3);"#;

        let tokens = Lexer::tokenize(src).expect("Couldnt tokenize input");

        assert(&tokens[0], "let", TokenType::LetKeyword);
        assert(&tokens[1], "x", TokenType::Identifier);
        assert(&tokens[2], "=", TokenType::Equals);
        assert(&tokens[3], "5", TokenType::Number);
        assert(&tokens[4], "+", TokenType::BinaryOperator);
        assert(&tokens[5], "(", TokenType::LeftParen);
        assert(&tokens[6], "4", TokenType::Number);
        assert(&tokens[7], "/", TokenType::BinaryOperator);
        assert(&tokens[8], "3", TokenType::Number);
        assert(&tokens[9], ")", TokenType::RightParen);
        assert(&tokens[10], ";", TokenType::Semicolon);
        assert(&tokens[11], "", TokenType::Eof);
    }
}
