use crate::error::lex_error::LexerError;
use crate::error::parse_error::ParserError;
use crate::error::runtime_error::RuntimeError;

// ================================================================
// COMPILE_ERROR
// ================================================================
//
// Un seul type, comme avant — AUCUN changement de signature nécessaire
// dans tout le reste du compilateur (compile_expression, compile_var,
// resolve_variable, etc. gardent tous `Result<_, CompileError>`).
//
// La position (ligne, colonne) est ajoutée via UNE SEULE variante
// enveloppante, `WithLocation`, posée UNE SEULE FOIS au tout dernier
// moment — juste avant que l'erreur ne s'échappe vers l'appelant, dans
// les 3 points d'entrée du compilateur (`compile`, `compile_module`,
// `compile_function`). Le reste du compilateur continue de construire et
// de propager des `CompileError` bruts via `?`, exactement comme avant :
// zéro ripple sur les ~40 signatures existantes.

#[derive(Debug)]
#[allow(dead_code)]
pub enum CompileError {
    VariableAlreadyDeclared(String),
    VariableUseInInitializer(String),
    UndefinedVariable {
        name: String,
        suggestion: Option<String>,
    },

    AssignmentToConstant(String),

    TypeMismatch {
        expected: String,
        found: String,
    },

    InvalidUnaryOperation {
        operator: String,
        found: String,
    },

    InvalidBinaryOperation {
        operator: String,
        left: String,
        right: String,
    },

    WrongArgumentType {
        function: String,
        index: usize,
        expected: String,
        found: String,
    },

    NotCallable {
        found: String,
    },

    TooManyConstants,
    TooManyArguments,
    TooManyArrayElements,
    TooManyLocals,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsidFunction,
    TooManyUpvalues,
    ExpectedDeclarationAfterExport,

    /// Erreurs de syntaxe rencontrées en parsant un module importé. On ne
    /// garde que la première pour l'affichage (voir `to_diagnostic`) — le
    /// parser continue après une erreur pour tenter de repérer d'autres
    /// problèmes, mais ces erreurs "en cascade" n'aident pas l'utilisateur
    /// et ne font que noyer la vraie cause. `path`/`source` sont le
    /// fichier et le texte du MODULE (pas celui qui l'importe), pour
    /// pouvoir afficher le bon extrait de code.
    ModuleParserErrors {
        path: String,
        source: String,
        errors: Vec<ParserError>,
    },
    ModuleLexerErrors {
        path: String,
        source: String,
        errors: Vec<LexerError>,
    },

    WrongArgumentCount {
        expected: i32,
        found: usize,
    },

    InvalidMemberAccess {
        name: String,
    },

    /// Incohérence interne du compilateur (ex. pile de boucles
    /// désynchronisée) — ne devrait jamais se produire si le compilateur
    /// est correct, mais on préfère un message d'erreur clair à un panic
    /// qui tue tout le processus si un futur bug en introduit une.
    InternalCompilerError(String),

    // ============================================================
    // MODULE / IMPORT / EXPORT
    // ============================================================
    DuplicateExport(String),

    CircularImport(String),

    ModuleNotFound(String),

    ModuleInvalidPath(String),

    ModuleReadError {
        path: String,
        message: String,
    },

    ModuleRuntimeError {
        path: String,
        module_source: String,
        source: RuntimeError,
    },

    /// Erreur de compilation À L'INTÉRIEUR d'un module importé (avant même
    /// son exécution) — porte le chemin ET le texte source du module
    /// concerné (`module_source`), pour qu'un diagnostic remonté depuis sa
    /// compilation affiche le bon fichier et le bon extrait de code,
    /// plutôt que ceux du fichier qui a déclenché l'import.
    #[allow(clippy::enum_variant_names)]
    ModuleCompileError {
        path: String,
        module_source: String,
        error: Box<CompileError>,
    },

    ExportNotFound {
        module: String,
        name: String,
    },

    InvalidExport,
    InvalidImport,

    /// Enveloppe posée une seule fois, au tout dernier moment, autour de
    /// n'importe quelle autre variante — voir le commentaire d'en-tête.
    WithLocation {
        line: usize,
        column: usize,
        source: Box<CompileError>,
    },
    InvalidJump,
    JumpTooLarge,
    TooManyObjectFields,
    TooManyTupleElements,

    DuplicateMethod {
        class_name: String,
        method_name: String,
        arity: usize,
    },

    /// Accès statique à un membre `private` depuis l'extérieur de la
    /// classe qui le déclare.
    PrivateMemberAccess {
        class_name: String,
        member: String,
    },

    ProtectedMemberAccess {
        class_name: String,
        member: String,
    },

    /// Méthode de collection supprimée par la standardisation de l'API
    /// (`length` -> `size()`, `push` -> `add(value)`...).
    RenamedMember {
        name: String,
        replacement: String,
    },

    /// Expression imbriquée trop profondément (protège la pile native du
    /// vérificateur de types et du compilateur, qui sont récursifs).
    ExpressionTooDeep {
        limit: usize,
    },

    /// Deux fonctions globales de même nom ET de même nombre de paramètres
    /// (la surcharge se fait par arité).
    DuplicateFunction {
        name: String,
        arity: usize,
    },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::WithLocation {
                line,
                column,
                source,
            } => {
                write!(f, "ligne {line}, colonne {column} : {source}")
            }

            CompileError::VariableAlreadyDeclared(e) => {
                write!(f, "Variable '{e}' déjà déclarée")
            }

            CompileError::VariableUseInInitializer(e) => {
                write!(f, "Variable '{e}' utilisée dans son propre initialiseur")
            }

            CompileError::UndefinedVariable { name, suggestion } => match suggestion {
                Some(suggestion) => write!(
                    f,
                    "Variable '{name}' non définie. Vouliez-vous dire '{suggestion}' ?"
                ),

                None => write!(f, "Variable '{name}' non définie"),
            },

            CompileError::AssignmentToConstant(e) => {
                write!(f, "Affectation à la constante '{e}'")
            }

            CompileError::TypeMismatch { expected, found } => {
                write!(
                    f,
                    "Type incompatible : '{found}' ne peut pas être utilisé comme '{expected}'"
                )
            }

            CompileError::InvalidUnaryOperation { operator, found } => {
                write!(
                    f,
                    "Opérateur '{operator}' invalide pour une valeur de type '{found}'"
                )
            }

            CompileError::InvalidBinaryOperation {
                operator,
                left,
                right,
            } => {
                write!(
                    f,
                    "Opérateur '{operator}' invalide entre '{left}' et '{right}'"
                )
            }

            CompileError::WrongArgumentType {
                function,
                index,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Argument {index} de '{function}' : '{found}' fourni, '{expected}' attendu"
                )
            }

            CompileError::NotCallable { found } => {
                write!(f, "La valeur de type '{found}' n'est pas appelable")
            }

            CompileError::TooManyConstants => {
                write!(f, "Trop de constantes dans ce fragment de code")
            }
            CompileError::TooManyArguments => {
                write!(f, "Trop d'arguments ou de paramètres")
            }
            CompileError::TooManyArrayElements => {
                write!(f, "Trop d'éléments dans le tableau")
            }
            CompileError::TooManyLocals => {
                write!(f, "Trop de variables locales dans cette fonction")
            }

            CompileError::BreakOutsideLoop => {
                write!(f, "'break' en dehors d'une boucle")
            }

            CompileError::ContinueOutsideLoop => {
                write!(f, "'continue' en dehors d'une boucle")
            }

            CompileError::ReturnOutsidFunction => {
                write!(f, "'return' en dehors d'une fonction")
            }

            CompileError::TooManyUpvalues => {
                write!(f, "Trop de variables capturées par cette closure")
            }

            CompileError::WrongArgumentCount { expected, found } => {
                write!(f, "{expected} argument(s) attendu(s), {found} fourni(s)")
            }

            CompileError::InvalidMemberAccess { name } => {
                write!(f, "Accès de membre invalide : '{name}'")
            }

            CompileError::InternalCompilerError(message) => {
                write!(f, "Erreur interne du compilateur : {message}")
            }

            // ====================================================
            // MODULE / IMPORT / EXPORT
            // ====================================================
            CompileError::DuplicateExport(name) => {
                write!(f, "L'export '{name}' est déjà déclaré")
            }

            CompileError::CircularImport(path) => {
                write!(f, "Import de module circulaire : {path}")
            }

            CompileError::ModuleNotFound(path) => {
                write!(f, "Module introuvable : {path}")
            }

            CompileError::ModuleInvalidPath(path) => {
                write!(f, "Chemin de module invalide : {path}")
            }

            CompileError::ModuleReadError { path, message } => {
                write!(f, "Impossible de lire le module '{path}' : {message}")
            }

            CompileError::ModuleRuntimeError { path, source, .. } => {
                write!(f, "Erreur d'exécution dans le module '{path}' : {source}")
            }

            CompileError::ModuleCompileError { path, error, .. } => {
                write!(f, "Erreur de compilation dans le module '{path}' : {error}")
            }

            CompileError::ExportNotFound { module, name } => {
                write!(f, "Le module '{module}' n'exporte pas '{name}'")
            }

            CompileError::InvalidExport => {
                write!(f, "Déclaration d'export invalide")
            }
            CompileError::InvalidImport => {
                write!(f, "Déclaration d'import invalide")
            }
            CompileError::ExpectedDeclarationAfterExport => {
                write!(f, "Déclaration attendue après 'export'")
            }
            CompileError::ModuleParserErrors { path, errors, .. } => {
                // On n'affiche que la première erreur : le parser continue
                // après une erreur pour tenter d'en repérer d'autres, mais
                // ces erreurs "en cascade" ne font que noyer la vraie cause
                // (voir aussi `KastelError::render` pour le même choix côté
                // fichier principal).
                match errors.first() {
                    Some(error) => write!(
                        f,
                        "Erreur de syntaxe dans le module '{path}' à {}:{} : {}",
                        error.line, error.column, error.message
                    ),
                    None => write!(f, "Erreur de syntaxe dans le module '{path}'"),
                }
            }
            CompileError::ModuleLexerErrors { path, errors, .. } => match errors.first() {
                Some(error) => write!(
                    f,
                    "Erreur lexicale dans le module '{path}' à {}:{} : {}",
                    error.line, error.column, error.message
                ),
                None => write!(f, "Erreur lexicale dans le module '{path}'"),
            },
            CompileError::InvalidJump => {
                write!(f, "Saut de bytecode invalide")
            }

            CompileError::JumpTooLarge => {
                write!(f, "Saut de bytecode trop grand")
            }
            CompileError::TooManyObjectFields => {
                write!(f, "Trop de champs dans cet objet")
            }
            CompileError::TooManyTupleElements => {
                write!(f, "Trop d'éléments dans le tuple")
            }

            CompileError::DuplicateMethod {
                class_name,
                method_name,
                arity,
            } => {
                write!(
                    f,
                    "La méthode '{class_name}.{method_name}' avec {arity} argument(s) est déjà déclarée"
                )
            }

            CompileError::PrivateMemberAccess { class_name, member } => {
                write!(
                    f,
                    "Le membre '{class_name}.{member}' est privé : accès refusé en dehors de la classe"
                )
            }

            CompileError::ProtectedMemberAccess { class_name, member } => {
                write!(
                    f,
                    "Le membre '{class_name}.{member}' est protégé : accès réservé à la classe et à ses classes dérivées"
                )
            }

            CompileError::RenamedMember { name, replacement } => {
                write!(f, "'{name}' n'existe plus : utilisez '{replacement}'")
            }

            CompileError::DuplicateFunction { name, arity } => {
                write!(
                    f,
                    "La fonction '{name}' avec {arity} argument(s) est déjà déclarée"
                )
            }

            CompileError::ExpressionTooDeep { limit } => {
                write!(
                    f,
                    "Expression trop profondément imbriquée (limite : {limit}) : scindez-la en plusieurs instructions"
                )
            }
        }
    }
}

impl CompileError {
    /// Équivalent de `RuntimeError::to_diagnostic` côté compilateur : voir
    /// les commentaires de cette dernière pour la philosophie générale
    /// (position récupérée depuis `WithLocation`, expected/found/help
    /// uniquement là où l'information existe réellement).
    pub fn to_diagnostic(&self) -> crate::error::diagnostic::Diagnostic {
        use crate::error::diagnostic::Diagnostic;

        if let CompileError::WithLocation {
            line,
            column,
            source,
        } = self
        {
            let mut diagnostic = source.to_diagnostic();

            // Si l'erreur interne pointe déjà vers un AUTRE fichier
            // (`with_source_file`, typiquement une erreur à l'intérieur
            // d'un module importé), on ne doit surtout pas écraser sa
            // position avec celle du `from ... import` dans le fichier
            // qui a déclenché l'import — sinon le diagnostic finirait par
            // afficher "main.ks:1:1" avec un extrait de code du module.
            // On ne prend la position englobante que pour une erreur qui
            // concerne réellement CE fichier-ci.
            if diagnostic.source_override.is_none() {
                diagnostic.line = *line;
                diagnostic.column = *column;
            }

            return diagnostic;
        }

        match self {
            CompileError::UndefinedVariable { name, suggestion } => {
                let diagnostic = Diagnostic::new(format!("variable '{name}' non définie"), 0, 0)
                    .with_len(name.len());

                match suggestion {
                    Some(suggestion) => {
                        diagnostic.with_help(format!("vouliez-vous dire '{suggestion}' ?"))
                    }
                    None => diagnostic,
                }
            }

            CompileError::VariableAlreadyDeclared(name) => Diagnostic::new(
                format!("la variable '{name}' est déjà déclarée dans cette portée"),
                0,
                0,
            )
            .with_len(name.len())
            .with_help(format!(
                "renommez l'une des deux déclarations, ou retirez `let`/`const` \
                 si vous vouliez réaffecter '{name}'."
            )),

            CompileError::VariableUseInInitializer(name) => Diagnostic::new(
                format!("'{name}' est utilisée dans son propre initialiseur"),
                0,
                0,
            )
            .with_len(name.len())
            .with_help(format!(
                "'{name}' n'existe pas encore au moment où son initialiseur s'exécute ; \
                 utilisez un autre nom pour la valeur dont vous avez besoin."
            )),

            CompileError::AssignmentToConstant(name) => Diagnostic::new(
                format!("impossible de réaffecter la constante '{name}'"),
                0,
                0,
            )
            .with_len(name.len())
            .with_expected("une variable déclarée avec 'let'")
            .with_found("une variable déclarée avec 'const'")
            .with_help(format!("déclarez '{name}' avec `let` au lieu de `const` si elle doit changer.")),

            CompileError::TypeMismatch { expected, found } => Diagnostic::new(
                "type incompatible",
                0,
                0,
            )
            .with_expected(expected.as_str())
            .with_found(found.as_str()),

            CompileError::InvalidUnaryOperation { operator, found } => Diagnostic::new(
                format!("opérateur '{operator}' invalide"),
                0,
                0,
            )
            .with_found(found.as_str()),

            CompileError::InvalidBinaryOperation { operator, left, right } => Diagnostic::new(
                format!("opérateur '{operator}' invalide"),
                0,
                0,
            )
            .with_expected(left.as_str())
            .with_found(right.as_str()),

            CompileError::WrongArgumentType { function, index, expected, found } => Diagnostic::new(
                format!("type incorrect pour l'argument {index} de '{function}'"),
                0,
                0,
            )
            .with_expected(expected.as_str())
            .with_found(found.as_str()),

            CompileError::NotCallable { found } => Diagnostic::new(
                "valeur non appelable",
                0,
                0,
            )
            .with_found(found.as_str()),

            CompileError::WrongArgumentCount { expected, found } => {
                Diagnostic::new("nombre d'arguments incorrect", 0, 0)
                    .with_expected(format!("{expected} argument(s)"))
                    .with_found(format!("{found} argument(s)"))
            }

            CompileError::DuplicateMethod {
                class_name,
                method_name,
                arity,
            } => Diagnostic::new(
                format!(
                    "la méthode '{class_name}.{method_name}' avec {arity} argument(s) est déjà déclarée"
                ),
                0,
                0,
            ),

            CompileError::PrivateMemberAccess { class_name, member } => Diagnostic::new(
                format!("le membre '{class_name}.{member}' est privé"),
                0,
                0,
            )
            .with_len(member.len())
            .with_help(
                "utilisez une méthode publique de la classe (par exemple un accesseur) au lieu d'accéder directement au membre.",
            ),

            CompileError::DuplicateFunction { name, arity } => Diagnostic::new(
                format!("la fonction '{name}' avec {arity} argument(s) est déjà déclarée"),
                0,
                0,
            )
            .with_len(name.len())
            .with_help(
                "une fonction peut être surchargée, mais seulement avec un NOMBRE de paramètres différent.",
            ),

            CompileError::ExpressionTooDeep { limit } => Diagnostic::new(
                format!("expression trop profondément imbriquée (limite : {limit})"),
                0,
                0,
            )
            .with_help("scindez l'expression en plusieurs instructions intermédiaires."),

            CompileError::ProtectedMemberAccess { class_name, member } => Diagnostic::new(
                format!("le membre '{member}' de la classe '{class_name}' est protégé"),
                0,
                0,
            )
            .with_len(member.len())
            .with_help(
                "utilisez ce membre depuis la classe qui le déclare ou depuis une classe dérivée.",
            ),

            CompileError::RenamedMember { name, replacement } => Diagnostic::new(
                format!("'{name}' a été remplacé par '{replacement}'"),
                0,
                0,
            )
            .with_len(name.len())
            .with_help(format!(
                "l'API des collections est standardisée : utilisez '{replacement}'."
            )),

            CompileError::InvalidMemberAccess { name } => {
                Diagnostic::new(format!("accès de membre invalide : '{name}'"), 0, 0)
                    .with_len(name.len())
            }

            CompileError::BreakOutsideLoop => {
                Diagnostic::new("'break' en dehors d'une boucle", 0, 0)
                    .with_len(5)
                    .with_help("'break' ne peut apparaître qu'à l'intérieur d'un 'while' ou d'un 'for'.")
            }

            CompileError::ContinueOutsideLoop => {
                Diagnostic::new("'continue' en dehors d'une boucle", 0, 0)
                    .with_len(8)
                    .with_help("'continue' ne peut apparaître qu'à l'intérieur d'un 'while' ou d'un 'for'.")
            }

            CompileError::ReturnOutsidFunction => {
                Diagnostic::new("'return' en dehors d'une fonction", 0, 0)
                    .with_len(6)
                    .with_help("'return' ne peut apparaître qu'à l'intérieur d'un 'func'.")
            }

            CompileError::TooManyUpvalues => Diagnostic::new(
                "trop de variables capturées par cette closure",
                0,
                0,
            )
            .with_help("réduisez le nombre de variables externes utilisées dans cette fonction imbriquée."),

            CompileError::ModuleNotFound(path) => {
                Diagnostic::new(format!("module introuvable : '{path}'"), 0, 0)
                    .with_len(path.len())
                    .with_help("vérifiez le chemin du module et son extension ('.ks').")
            }

            // ------------------------------------------------------------
            // Erreurs À L'INTÉRIEUR d'un module importé : on ne garde que
            // la première (pas de cascade — voir le commentaire sur le
            // Display de ces variantes), et surtout on bascule le
            // diagnostic sur le fichier/texte du MODULE via
            // `with_source_file`, pour que `-->` et l'extrait de code
            // pointent dans le bon fichier plutôt que dans celui qui a
            // fait l'import.
            // ------------------------------------------------------------
            CompileError::ModuleParserErrors {
                path,
                source,
                errors,
            } => match errors.first() {
                Some(error) => Diagnostic::new(error.message.clone(), error.line, error.column)
                    .with_source_file(path, source),

                None => Diagnostic::new(format!("erreur de syntaxe dans le module '{path}'"), 0, 0)
                    .with_source_file(path, source),
            },

            CompileError::ModuleLexerErrors {
                path,
                source,
                errors,
            } => match errors.first() {
                Some(error) => Diagnostic::new(error.message.clone(), error.line, error.column)
                    .with_source_file(path, source),

                None => Diagnostic::new(format!("erreur lexicale dans le module '{path}'"), 0, 0)
                    .with_source_file(path, source),
            },

            CompileError::ModuleCompileError {
                path,
                module_source,
                error,
            } => error
                .to_diagnostic()
                .with_source_file(path, module_source),

            CompileError::ModuleRuntimeError {
                path,
                module_source,
                source,
            } => source
                .to_diagnostic()
                .with_source_file(path, module_source),

            CompileError::ExportNotFound { module, name } => Diagnostic::new(
                format!("le module '{module}' n'exporte pas '{name}'"),
                0,
                0,
            )
            .with_len(name.len()),

            CompileError::CircularImport(path) => {
                Diagnostic::new(format!("import de module circulaire : '{path}'"), 0, 0)
                    .with_help("un module ne peut pas s'importer lui-même, directement ou indirectement.")
            }

            // Variantes sans donnée exploitable pour expected/found/help :
            // on garde un titre honnête (dérivé de Display) plutôt que
            // d'inventer une information qu'on n'a pas.
            other => Diagnostic::new(other.to_string(), 0, 0),
        }
    }
}
