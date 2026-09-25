#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParam {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// Nom simple : `int`, `Personne`, `Dict`, ...
    Named(String),

    /// Type paramétré : `Dict<str, int>`, `List<Personne>`, `Result<T, E>`.
    Generic {
        name: String,
        arguments: Vec<TypeExpr>,
    },

    /// Union : `int | float`.
    Union(Vec<TypeExpr>),

    /// Type objet (enregistrement) : `{ name: str, age: int }`.
    Record(Vec<(String, TypeExpr)>),
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
/// Nom de la méthode-constructeur : `func initialize(...)`.
///
/// Elle peut être surchargée par arité. Si une classe n'en déclare aucune,
/// un constructeur par défaut implicite (sans paramètre) est utilisé, et une
/// classe dérivée hérite des constructeurs de sa classe de base.
pub const CONSTRUCTOR_NAME: &str = "initialize";

/// Ancien nom du constructeur, désormais refusé avec un message de migration.
pub const LEGACY_CONSTRUCTOR_NAME: &str = "init";

/// Préfixe de la méthode cachée qui porte les valeurs initiales des champs
/// d'une classe (`__fields_<Classe>`). La VM l'exécute à la création de
/// chaque instance, AVANT le constructeur, en commençant par la classe de
/// base.
pub const FIELD_INITIALIZER_PREFIX: &str = "__fields_";

/// Visibilité d'un membre de classe (champ ou méthode).
///
/// `Public` est la valeur par défaut. `Protected` autorise l'accès depuis
/// la classe qui déclare le membre et depuis ses classes dérivées. `Private`
/// limite l'accès à la classe qui déclare le membre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

/// Champ déclaré dans le corps d'une classe :
/// `private let age: int = 0;`
#[derive(Debug, Clone)]
pub struct ClassField {
    pub name: String,
    pub visibility: Visibility,
    /// `static let compteur: int = 0;` : porté par la CLASSE elle-même
    /// (une seule valeur partagée, accessible via `NomClasse.compteur`),
    /// et non par chaque instance. Voir `FunctionMethod::is_static`.
    pub is_static: bool,
    /// Annotation de type (`: int`), `None` = champ dynamique.
    pub type_annotation: Option<TypeExpr>,
    /// Valeur initiale. Pour un champ d'INSTANCE, le parser la transforme
    /// en méthode cachée `__fields_<Classe>`, exécutée par la VM à chaque
    /// `new`, avant le constructeur (voir `FIELD_INITIALIZER_PREFIX`).
    /// Pour un champ STATIQUE, elle est évaluée une seule fois, à la
    /// déclaration de la classe (voir `compile_class`).
    pub initializer: Option<Expression>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionMethod {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub visibility: Visibility,
    /// `static func creer(...) { ... }` : appelée sur la CLASSE
    /// (`NomClasse.creer(...)`), SANS `this` implicite — contrairement à
    /// une méthode normale, elle ne reçoit pas d'instance. Une méthode
    /// statique ne peut donc pas utiliser `this` ni `base`.
    pub is_static: bool,
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
    pub generic_params: Vec<GenericParam>,
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
        generic_params: Vec<GenericParam>,
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
    /// `type Person = { name: str, age: int };` : alias de type (compile
    /// uniquement, sans effet à l'exécution).
    TypeAlias {
        name: String,
        generic_params: Vec<GenericParam>,
        type_expr: TypeExpr,
    },

    Class {
        name: String,
        generic_params: Vec<GenericParam>,
        bases: Vec<TypeExpr>,
        fields: Vec<ClassField>,
        methods: Vec<FunctionMethod>,
    },

    /// Enum à variants nommés : `enum Color { Red, Green, Blue }`.
    /// Les variants sont accessibles uniquement sous la forme `Color.Red`.
    Enum {
        name: String,
        generic_params: Vec<GenericParam>,
        variants: Vec<String>,
        methods: Vec<FunctionMethod>,
    },

    Interface {
        name: String,
        generic_params: Vec<GenericParam>,
        bases: Vec<TypeExpr>,
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
        generic_args: Vec<TypeExpr>,
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
        generic_args: Vec<TypeExpr>,
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

    /// Dict : `{"name": "Bruno", "age": 25}` — clés CHAÎNES, ajout et
    /// retrait dynamiques, accès par `d["name"]` / `d.get("name")`.
    Dict(Vec<(String, Expression)>),

    /// Record : `{ name: "Bruno", age: 25 }` — clés IDENTIFIANTS, forme
    /// fixe, accès par `p.name`.
    Record(Vec<(String, Expression)>),

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
