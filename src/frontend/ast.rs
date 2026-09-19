#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// Nom simple : `int`, `Personne`, `Dict`, ...
    Named(String),

    /// Type paramétré : `Dict<str, int>`, `Array<Personne>`, `Result<T, E>`.
    Generic {
        name: String,
        arguments: Vec<TypeExpr>,
    },
}

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
    /// Annotations de type des paramètres, un slot par entrée de
    /// `params` (même longueur, même ordre). `None` = paramètre non
    /// annoté et donc dynamique.
    pub param_types: Vec<Option<TypeExpr>>,
    /// Type de retour annoté (`-> Type`), `None` si omis.
    pub return_type: Option<TypeExpr>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub arity: usize,
    pub params: Vec<String>,
    pub param_types: Vec<Option<TypeExpr>>,
    pub return_type: Option<TypeExpr>,
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
        /// Annotation de type explicite (`let x: int = ...`), `None`
        /// si omise : le type est inféré statiquement lorsque c'est
        /// possible, sinon il devient dynamique.
        type_annotation: Option<TypeExpr>,
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
        /// Voir `FunctionMethod::param_types`.
        param_types: Vec<Option<TypeExpr>>,
        /// Voir `FunctionMethod::return_type`.
        return_type: Option<TypeExpr>,
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
        bases: Vec<String>,
        methods: Vec<FunctionMethod>,
    },
    Interface {
        name: String,
        bases: Vec<String>,
        methods: Vec<InterfaceMethod>,
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
    None,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),

    Variable(String),

    Unary {
        operator: UnaryOp,
        right: Box<Expression>,
        /// Position du début de l'opérande (`right`), pour les diagnostics
        /// d'erreur runtime (ex. `-"abc"`).
        line: usize,
        column: usize,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
        /// Position du début de l'opérande droit (`right`) — c'est celle
        /// utilisée par le compilateur pour positionner précisément les
        /// erreurs runtime de type (ex. `expected/found` sur `name + age`
        /// pointe sous `age`), plutôt que sur le début de l'instruction
        /// entière comme c'était le cas auparavant.
        line: usize,
        column: usize,
    },

    Function {
        params: Vec<String>,
        body: Vec<Statement>,
    },

    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        /// Position du `(` d'appel — utilisée pour les diagnostics runtime
        /// (arité incorrecte, valeur non appelable...) : à défaut de
        /// pouvoir pointer sous le nom de la fonction appelée (les
        /// expressions comme `Variable` ne portent pas de position), on
        /// pointe juste après elle, au début de la liste d'arguments.
        line: usize,
        column: usize,
    },

    Member {
        object: Box<Expression>,
        name: String,
        /// Position du nom de membre lui-même (après le `.`), pour
        /// pointer précisément sous le champ fautif (ex. `obj.inexistant`).
        line: usize,
        column: usize,
    },

    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        /// Position du début de l'expression d'index (entre `[` et `]`),
        /// pour pointer sous l'index fautif (ex. `arr[10]`).
        line: usize,
        column: usize,
    },
    New {
        class_name: String,
        arguments: Vec<Expression>,
        /// Position du nom de la classe instanciée.
        line: usize,
        column: usize,
    },

    This,
    Base,
    Array(Vec<Expression>),

    /// `(a, b, c)`, `(a,)` (tuple à un élément), `()` (tuple vide).
    /// Se distingue d'une simple expression parenthésée `(a)` par la
    /// virgule : voir `Parser::primary` pour la règle de désambiguïsation.
    Tuple(Vec<Expression>),

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
    Is,

    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}
