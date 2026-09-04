#[derive(Debug, Clone)]
pub enum AssignmentTarget {
    Variable(String),

    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    Member {
        object: Box<Expression>,
        name: String,
    },
}
#[derive(Debug, Clone)]
pub struct ModulePath {
    pub parts: Vec<String>,
}

impl ModulePath {
    pub fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }

    // pub fn as_string(&self) -> String {
    //     self.parts.join(".")
    // }

    // pub fn last(&self) -> Option<&str> {
    //     self.parts.last().map(String::as_str)
    // }
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Statement {
    /// Enveloppe posée systématiquement par `Parser::statement()` autour de
    /// CHAQUE statement produit, quel que soit son type. Permet au
    /// compilateur de connaître la position source exacte d'un statement
    /// (pour remplir `chunk.lines`/`chunk.columns` et enrichir les erreurs
    /// de compilation) SANS avoir à ajouter un champ `line`/`column` à
    /// chacune des variantes ci-dessous — un seul point de câblage dans le
    /// parser (statement()) et un seul dans le compilateur
    /// (compile_statement), plutôt que des dizaines.
    Positioned {
        // line: usize,
        // column: usize,
        statement: Box<Statement>,
    },

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

    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    While {
        condition: Expression,
        body: Vec<Statement>,
    },

    ForIn {
        variable: String,
        iterable: Expression,
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
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]

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

    /// Littéral d'objet : { clé: expression, ... }
    /// L'ordre des champs est préservé (Vec, pas HashMap) pour que
    /// l'affichage et l'itération future respectent l'ordre d'écriture.
    Object(Vec<(String, Expression)>),

    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
    BitNot,
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

    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}