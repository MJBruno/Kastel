#[derive(Debug, Clone)]
pub enum AssignmentTarget {
    Variable(String),

    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
}
#[derive(Debug, Clone)]

pub struct ModulePath {
    pub parts: Vec<String>,
}
#[allow(dead_code)]
impl ModulePath {
    pub fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }

    pub fn as_string(&self) -> String {
        self.parts.join(".")
    }

    pub fn last(&self) -> Option<&str> {
        self.parts.last().map(String::as_str)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
        mutable: bool,
    },

    Assignment {
        target: AssignmentTarget,
        value: Expression,
    },

    Expression {
        expression: Expression,
    },

    Block(Vec<Statement>),

    Print(Expression),

    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    While {
        condition: Expression,
        body: Vec<Statement>,
    },

    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },

    Return {
        value: Option<Expression>,
    },
    Import {
        path: Vec<String>,
    },
    FromImport {
        module: ModulePath,
        items: Vec<ImportItem>,
    },

    Export {
        statement: Box<Statement>,
    },

    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expression {
    Literal(Literal),

    Variable(String),

    Unary {
        operator: UnaryOp,
        right: Box<Expression>,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },

    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },

    Member {
        object: Box<Expression>,
        name: String,
    },

    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    Array(Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,

    Less,
    LessEqual,

    Greater,
    GreaterEqual,

    And,
    Or,
}
