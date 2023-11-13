#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub value: String,
    pub token_type: TokenType,
}

impl Token {
    pub fn new(value: impl Into<String>, token_type: TokenType) -> Self {
        Self {
            value: value.into(),
            token_type,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    // foo_bar
    Identifier,

    // 0-9
    Number,

    // + - / * %
    BinaryOperator,

    // Comment
    Comment,

    // {
    LeftBrace,

    // }
    RightBrace,

    // [
    LeftBracket,

    // ]
    RightBracket,

    // (
    LeftParen,

    // )
    RightParen,

    // :
    Colon,

    // ;
    Semicolon,

    // ,
    Comma,

    // =
    Equals,

    // .
    Dot,

    // fn
    FnKeyword,

    // struct
    StructKeyword,

    // enum
    EnumKeyword,

    // let
    LetKeyword,

    // const
    ConstKeyword,

    // return
    ReturnKeyword,

    // if
    IfKeyword,

    // else
    ElseKeyword,

    // EOF
    Eof,
}

impl TokenType {
    pub fn from_keyword(keyword: &str) -> Option<TokenType> {
        let token_type = match keyword {
            "let" => TokenType::LetKeyword,
            "const" => TokenType::ConstKeyword,
            "fn" => TokenType::FnKeyword,
            _ => {
                return None;
            }
        };

        Some(token_type)
    }
}
