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
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

// ============================================================
// PATTERN MATCHING
// ============================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Pattern {
    /// `_`
    Wildcard,

    /// `x`
    Binding(String),

    /// `1`, `"hello"`, true, false, null
    Literal(Literal),

    /// `1 | 2 | 3`
    Or(Vec<Pattern>),

    /// `1 .. 10`
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },

    /// `[x, y, _]`
    Array(Vec<Pattern>),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone)]
pub struct FunctionMethod {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone)]
pub enum Statement {
    Positioned {
        line: usize,
        column: usize,
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

    // ========================================================
    // MATCH
    // ========================================================
    Match {
        value: Expression,
        arms: Vec<MatchArm>,
    },

    // ========================================================
    // EXCEPTIONS
    // ========================================================
    /// `throw expression;`
    Throw {
        value: Expression,
    },

    ///
    /// try {
    ///     ...
    /// } catch (error) {
    ///     ...
    /// } finally {
    ///     ...
    /// }
    ///
    /// `catch` est optionnel si `finally` est présent.
    Try {
        try_body: Vec<Statement>,
        catch_name: Option<String>,
        catch_body: Option<Vec<Statement>>,
        finally_body: Option<Vec<Statement>>,
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
    Class {
        name: String,
        superclass: Option<String>,
        methods: Vec<FunctionMethod>,
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

    Function {
        params: Vec<String>,
        body: Vec<Statement>,
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
    New {
        class_name: String,
        arguments: Vec<Expression>,
    },

    This,
    Base,
    Array(Vec<Expression>),

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
