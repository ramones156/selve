

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Program(Program),
    ObjectLiteral(Vec<Property>),
    NumericLiteral(String),
    Identifier(String),
    Comment(String),
    FnDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
        is_const: bool,
    },
    VarDeclaration {
        constant: bool,
        identifier: String,
        value: Option<Box<Stmt>>,
    },
    AssignmentExpr {
        assignee: Box<Stmt>,
        value: Box<Stmt>,
    },
    MemberExpr {
        object: Box<Stmt>,
        property: Box<Stmt>,
        computed: bool,
    },
    CallExpr {
        args: Vec<Stmt>,
        caller: Box<Stmt>,
    },
    BinaryExpr {
        left: Box<Stmt>,
        right: Box<Stmt>,
        operator: String,
    },
    UniaryExpr,
    FunctionDeclaration,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {}

#[derive(Debug, PartialEq, Clone)]
pub struct Property {
    pub key: String,
    pub value: Option<Box<Stmt>>,
}
