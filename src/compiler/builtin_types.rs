//! Contrats statiques des fonctions natives et intrinsèques de Kastel.
//!
//! Ce module est la source de vérité des contrats de la surface native :
//! noms, signatures et mode d'enregistrement.
//!
//! Les implémentations Rust restent dans `stdlib::*` et le runtime conserve
//! la responsabilité d'enregistrer les pointeurs de fonction. En revanche,
//! le compilateur ne maintient plus une seconde liste de noms :
//! `stdlib::register_compiler_natives` dérive directement de cette table.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::types::{FunctionType, GenericConstraint, Type};

/// Mode d'exposition d'un symbole natif.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeKind {
    /// Fonction réellement enregistrée dans les globals du runtime.
    Runtime,
    /// Fonction intrinsèque compilée directement en bytecode.
    Intrinsic(Intrinsic),
}

/// Sémantique spéciale d'appel d'une native.
///
/// La plupart des natives suivent leur `Type::Function` tel quel. Quelques
/// fonctions historiques ont toutefois une surface à arité variable ou un
/// résultat dépendant des arguments. Leur comportement particulier est déclaré
/// ici afin d'éviter que le TypeChecker ne traite leurs noms comme des exceptions
/// indépendantes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCallBehavior {
    /// Appel standard : la signature/les règles générales suffisent.
    Standard,
    /// `Set(a, b, ...)` déduit `Set<T>` à partir des éléments fournis.
    SetConstructor,
    /// `channel()` et `channel(capacity)` déduisent `Channel<T>`.
    ChannelConstructor,
    /// `range(...)` retourne toujours le type statique `Range`.
    RangeConstructor,
}

/// Cardinalité d'appel d'une fonction native ou intrinsèque.
///
/// `FunctionType` décrit les types des paramètres lorsqu'une signature fixe
/// est possible. `Arity` complète ce modèle pour les API dont le nombre
/// d'arguments varie sans pour autant rendre leur contrat statique
/// complètement dynamique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    Exact(usize),
    Range { min: usize, max: usize },
    AtLeast(usize),
}

impl Arity {
    pub const fn accepts(self, found: usize) -> bool {
        match self {
            Self::Exact(expected) => found == expected,
            Self::Range { min, max } => found >= min && found <= max,
            Self::AtLeast(min) => found >= min,
        }
    }

    /// Valeur historique à utiliser avec `WrongArgumentCount`.
    /// Pour une plage, on fournit la borne la plus proche de la valeur
    /// invalide afin de conserver l'erreur existante sans inventer une
    /// nouvelle représentation de diagnostic.
    pub const fn expected_for(self, found: usize) -> usize {
        match self {
            Self::Exact(expected) => expected,
            Self::Range { min, max } => {
                if found < min {
                    min
                } else {
                    max
                }
            }
            Self::AtLeast(min) => min,
        }
    }
}

/// Intrinsèques du langage pris en charge directement par le compilateur/VM.
///
/// Leur nom et leur identité sémantique sont centralisés ici afin d'éviter de
/// dupliquer des chaînes magiques dans le compilateur et le vérificateur de types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    Spawn,
    Yield,
    Sleep,
    Select,
}
#[allow(dead_code)]
impl Intrinsic {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Spawn => "spawn",
            Self::Yield => "yield",
            Self::Sleep => "sleep",
            Self::Select => "select",
        }
    }

    pub const fn all() -> [Self; 4] {
        [Self::Spawn, Self::Yield, Self::Sleep, Self::Select]
    }
}

pub fn intrinsic_kind(name: &str) -> Option<Intrinsic> {
    match name {
        "spawn" => Some(Intrinsic::Spawn),
        "yield" => Some(Intrinsic::Yield),
        "sleep" => Some(Intrinsic::Sleep),
        "select" => Some(Intrinsic::Select),
        _ => None,
    }
}

/// Contrat complet d'une fonction native/intrinsèque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSpec {
    pub name: &'static str,
    pub ty: Type,
    pub kind: NativeKind,
    pub arity: Arity,
    pub behavior: NativeCallBehavior,
}

fn function(params: &[Type], return_type: Type) -> Type {
    Type::Function(FunctionType {
        generic_params: Vec::new(),
        is_async: false,
        generic_constraints: Vec::<(String, Vec<GenericConstraint>)>::new(),
        params: params.to_vec(),
        return_type: Box::new(return_type),
    })
}

fn unary(argument: Type, result: Type) -> Type {
    function(&[argument], result)
}

fn binary(left: Type, right: Type, result: Type) -> Type {
    function(&[left, right], result)
}

fn generic_function(params: &[Type], return_type: Type, generic_params: &[&str]) -> Type {
    Type::Function(FunctionType {
        generic_params: generic_params
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        is_async: false,
        generic_constraints: Vec::new(),
        params: params.to_vec(),
        return_type: Box::new(return_type),
    })
}

fn runtime(name: &'static str, ty: Type) -> NativeSpec {
    let arity = match &ty {
        Type::Function(signature) => Arity::Exact(signature.params.len()),
        _ => panic!(
            "la native `{name}` doit fournir une arité explicite via runtime_with_arity"
        ),
    };

    NativeSpec {
        name,
        ty,
        kind: NativeKind::Runtime,
        arity,
        behavior: NativeCallBehavior::Standard,
    }
}

fn runtime_with_arity(name: &'static str, ty: Type, arity: Arity) -> NativeSpec {
    NativeSpec {
        name,
        ty,
        kind: NativeKind::Runtime,
        arity,
        behavior: NativeCallBehavior::Standard,
    }
}

fn runtime_with_behavior(
    name: &'static str,
    ty: Type,
    arity: Arity,
    behavior: NativeCallBehavior,
) -> NativeSpec {
    NativeSpec {
        name,
        ty,
        kind: NativeKind::Runtime,
        arity,
        behavior,
    }
}

fn intrinsic(kind: Intrinsic, ty: Type) -> NativeSpec {
    let arity = match &ty {
        Type::Function(signature) => Arity::Exact(signature.params.len()),
        _ => panic!(
            "l'intrinsèque `{}` doit fournir une arité explicite via intrinsic_with_arity",
            kind.name()
        ),
    };

    NativeSpec {
        name: kind.name(),
        ty,
        kind: NativeKind::Intrinsic(kind),
        arity,
        behavior: NativeCallBehavior::Standard,
    }
}

fn intrinsic_with_arity(kind: Intrinsic, ty: Type, arity: Arity) -> NativeSpec {
    NativeSpec {
        name: kind.name(),
        ty,
        kind: NativeKind::Intrinsic(kind),
        arity,
        behavior: NativeCallBehavior::Standard,
    }
}

/// Liste canonique des contrats natifs de Kastel.
///
/// Cette liste doit rester la seule déclaration statique des noms de natives.
/// Les modules de `stdlib` contiennent uniquement les implémentations Rust.
fn build_specs() -> Vec<NativeSpec> {
    use Type::*;

    let mut specs = Vec::new();

    // I/O. Les types d'arguments restent dynamiques, mais l'arité fait partie
    // du contrat statique et est contrôlée avant l'exécution.
    specs.push(runtime_with_arity("print", Dynamic, Arity::AtLeast(1)));
    specs.push(runtime_with_arity("println", Dynamic, Arity::AtLeast(1)));
    specs.push(runtime_with_arity("input", Dynamic, Arity::Range { min: 0, max: 1 }));

    // `Set(a, b, c)` : arité variable. Le TypeChecker déduit le type
    // `Set<T>` du résultat à partir des arguments.
    specs.push(runtime_with_behavior(
        "Set",
        Dynamic,
        Arity::AtLeast(0),
        NativeCallBehavior::SetConstructor,
    ));

    // Concurrence. Ces quatre symboles sont des intrinsèques du VM et ne sont
    // donc pas enregistrés comme NativeFunction dans les globals runtime.
    specs.push(intrinsic_with_arity(Intrinsic::Spawn, Dynamic, Arity::AtLeast(1)));
    // `channel()` est non borné, `channel(n)` est borné à `n` places : l'arité
    // est donc une plage, pas `Exact(0)` déduit de la signature sans paramètre.
    specs.push(runtime_with_behavior(
        "channel",
        function(
            &[],
            Generic {
                name: "Channel".into(),
                arguments: vec![Dynamic],
            },
        ),
        Arity::Range { min: 0, max: 1 },
        NativeCallBehavior::ChannelConstructor,
    ));
    specs.push(runtime("mutex", function(&[], Named("Mutex".into()))));
    specs.push(runtime(
        "semaphore",
        function(&[Int], Named("Semaphore".into())),
    ));
    specs.push(runtime(
        "wait_group",
        function(&[], Named("WaitGroup".into())),
    ));
    specs.push(runtime(
        "barrier",
        function(&[Int], Named("Barrier".into())),
    ));
    specs.push(runtime("rwlock", function(&[], Named("RwLock".into()))));
    specs.push(runtime("event", function(&[], Named("Event".into()))));
    specs.push(runtime(
        "condvar",
        function(&[Named("Mutex".into())], Named("Condvar".into())),
    ));
    specs.push(intrinsic(Intrinsic::Yield, function(&[], None)));
    specs.push(intrinsic(Intrinsic::Sleep, function(&[Int], None)));
    specs.push(intrinsic_with_arity(
        Intrinsic::Select,
        Dynamic,
        Arity::Range { min: 1, max: 2 },
    ));

    // Option / Result.
    specs.push(runtime(
        "Some",
        generic_function(
            &[Type::TypeParam("T".into())],
            Generic {
                name: "Option".into(),
                arguments: vec![Type::TypeParam("T".into())],
            },
            &["T"],
        ),
    ));
    specs.push(runtime(
        "Ok",
        generic_function(
            &[Type::TypeParam("T".into())],
            Generic {
                name: "Result".into(),
                arguments: vec![Type::TypeParam("T".into()), Dynamic],
            },
            &["T"],
        ),
    ));
    specs.push(runtime(
        "Err",
        generic_function(
            &[Type::TypeParam("E".into())],
            Generic {
                name: "Result".into(),
                arguments: vec![Dynamic, Type::TypeParam("E".into())],
            },
            &["E"],
        ),
    ));

    // Conversion / système.
    specs.push(runtime("int", unary(Dynamic, Int)));
    specs.push(runtime("float", unary(Dynamic, Float)));
    specs.push(runtime("str", unary(Dynamic, Str)));
    specs.push(runtime("bool", unary(Dynamic, Bool)));
    specs.push(runtime("type", unary(Dynamic, Str)));
    specs.push(runtime("clock", function(&[], Float)));
    specs.push(runtime("cwd", function(&[], Str)));
    specs.push(runtime("env", unary(Str, Dynamic)));

    // Math natives. Le runtime accepte int ou float via `expect_number` pour
    // les opérations numériques génériques ; les fonctions trigonométriques
    // renvoient toujours Float.
    specs.push(runtime("abs", unary(Dynamic, Dynamic)));
    specs.push(runtime("floor", unary(Dynamic, Int)));
    specs.push(runtime("ceil", unary(Dynamic, Int)));
    specs.push(runtime("round", unary(Dynamic, Int)));
    specs.push(runtime("sqrt", unary(Float, Float)));
    specs.push(runtime("pow", binary(Dynamic, Dynamic, Dynamic)));
    specs.push(runtime("min", binary(Dynamic, Dynamic, Dynamic)));
    specs.push(runtime("max", binary(Dynamic, Dynamic, Dynamic)));

    for name in [
        "sin", "cos", "tan", "asin", "acos", "atan", "exp", "log", "log10",
    ] {
        specs.push(runtime(name, unary(Float, Float)));
    }

    specs.push(runtime("atan2", binary(Float, Float, Float)));
    specs.push(runtime("rand", function(&[], Float)));
    specs.push(runtime("rand_int", unary(Dynamic, Int)));
    specs.push(runtime("rand_range", binary(Dynamic, Dynamic, Int)));

    // Arithmétique cyclique explicite.
    specs.push(runtime("wrapping_add", binary(Int, Int, Int)));
    specs.push(runtime("wrapping_sub", binary(Int, Int, Int)));
    specs.push(runtime("wrapping_mul", binary(Int, Int, Int)));

    // Division entière exacte, arrondie vers -infini.
    specs.push(runtime("idiv", binary(Dynamic, Dynamic, Int)));

    // Structures / utilitaires.
    specs.push(runtime(
        "dict",
        function(&[], Dict(Box::new(Dynamic), Box::new(Dynamic))),
    ));
    // `range` accepte 1, 2 ou 3 arguments. Son type d'élément reste dynamique
    // ici, mais sa cardinalité est entièrement connue statiquement.
    specs.push(runtime_with_behavior(
        "range",
        Dynamic,
        Arity::Range { min: 1, max: 3 },
        NativeCallBehavior::RangeConstructor,
    ));
    specs.push(runtime("list", unary(Dynamic, ArrayDynamic)));

    // Debug.
    specs.push(runtime("inspect", unary(Dynamic, Str)));
    specs.push(runtime("debug", unary(Dynamic, Dynamic)));
    specs.push(runtime_with_arity("format", Dynamic, Arity::AtLeast(1)));

    // JSON.
    specs.push(runtime("json_encode", unary(Dynamic, Str)));
    specs.push(runtime("json_decode", unary(Str, Dynamic)));

    // Fichiers.
    specs.push(runtime("file_read", unary(Str, Str)));
    specs.push(runtime("file_read_lines", unary(Str, Array(Box::new(Str)))));
    specs.push(runtime("file_write", function(&[Str, Str], None)));
    specs.push(runtime("file_append", function(&[Str, Str], None)));
    specs.push(runtime("file_exists", unary(Str, Bool)));
    specs.push(runtime("file_delete", unary(Str, None)));
    specs.push(runtime("file_size", unary(Str, Int)));

    // Path.
    specs.push(runtime("path_join", unary(Dynamic, Str)));
    specs.push(runtime("path_exists", unary(Str, Bool)));
    specs.push(runtime("path_is_dir", unary(Str, Bool)));
    specs.push(runtime("path_is_file", unary(Str, Bool)));
    specs.push(runtime("path_absolute", unary(Str, Str)));
    specs.push(runtime("path_basename", unary(Str, Str)));
    specs.push(runtime("path_dirname", unary(Str, Str)));
    specs.push(runtime("path_extension", unary(Str, Str)));
    specs.push(runtime("path_stem", unary(Str, Str)));

    // OS.
    specs.push(runtime("os_name", function(&[], Str)));
    specs.push(runtime("os_arch", function(&[], Str)));
    specs.push(runtime("args", function(&[], Array(Box::new(Str)))));
    specs.push(runtime_with_arity(
        "exit",
        Dynamic,
        Arity::Range { min: 0, max: 1 },
    ));

    // Processus externes. Le second argument accepte Array<str> ou Tuple<str>
    // au runtime ; Dynamic représente correctement cette surface polymorphe
    // tant que Kastel ne possède pas de type séquence commun.
    specs.push(runtime(
        "process_run",
        function(
            &[Str, Dynamic],
            Record(vec![
                ("stdout".into(), Str),
                ("stderr".into(), Str),
                ("code".into(), Int),
                ("success".into(), Bool),
            ]),
        ),
    ));

    specs
}

/// Contrats sous la forme historique attendue par le TypeChecker.

static NATIVE_REGISTRY: OnceLock<Vec<NativeSpec>> = OnceLock::new();

fn registry() -> &'static [NativeSpec] {
    NATIVE_REGISTRY
        .get_or_init(build_specs)
        .as_slice()
}

pub fn all() -> HashMap<String, Type> {
    registry()
        .iter()
        .map(|spec| (spec.name.to_string(), spec.ty.clone()))
        .collect()
}

/// Retourne le contrat natif complet correspondant au nom donné.
pub fn spec(name: &str) -> Option<&'static NativeSpec> {
    registry().iter().find(|spec| spec.name == name)
}

/// Retourne une vue possédant les contrats natifs/intrinsèques.
///
/// Cette API conserve la compatibilité avec les tests et modules qui ont
/// besoin d'itérer sur l'ensemble de la registry sans exposer directement
/// son stockage interne.
#[allow(dead_code)]
pub fn specs() -> Vec<NativeSpec> {
    registry().to_vec()
}

/// Retourne uniquement la cardinalité statique d'une native/intrinsèque.
pub fn native_arity(name: &str) -> Option<Arity> {
    spec(name).map(|spec| spec.arity)
}

/// Noms des fonctions effectivement enregistrées dans `Compiler` comme
/// globals natifs. Les intrinsèques du VM sont volontairement exclus.
pub fn compiler_native_names() -> impl Iterator<Item = &'static str> {
    registry()
        .iter()
        .filter(|spec| matches!(spec.kind, NativeKind::Runtime))
        .map(|spec| spec.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_math_signatures() {
        let types = all();

        assert_eq!(
            types.get("sin"),
            Some(&Type::Function(FunctionType {
                generic_params: Vec::new(),
                is_async: false,
                generic_constraints: Vec::new(),
                params: vec![Type::Float],
                return_type: Box::new(Type::Float),
            }))
        );

        assert_eq!(
            types.get("floor"),
            Some(&Type::Function(FunctionType {
                generic_params: Vec::new(),
                is_async: false,
                generic_constraints: Vec::new(),
                params: vec![Type::Dynamic],
                return_type: Box::new(Type::Int),
            }))
        );
    }

    #[test]
    fn all_native_contracts_have_unique_names_and_match_all() {
        let specs = registry();
        let all_types = all();
        let mut names = std::collections::HashSet::new();

        for spec in specs {
            assert!(names.insert(spec.name), "native dupliquée: {}", spec.name);
            assert_eq!(all_types.get(spec.name), Some(&spec.ty));
            let representative = spec.arity.expected_for(0);
            assert!(spec.arity.accepts(representative));
        }

        assert_eq!(names.len(), all_types.len());
    }

    #[test]
    fn intrinsic_identity_is_centralized() {
        for intrinsic in Intrinsic::all() {
            assert_eq!(intrinsic_kind(intrinsic.name()), Some(intrinsic));
        }
    }

    #[test]
    fn native_call_behaviors_are_centralized() {
        assert_eq!(
            spec("Set").map(|native| native.behavior),
            Some(NativeCallBehavior::SetConstructor)
        );
        assert_eq!(
            spec("channel").map(|native| native.behavior),
            Some(NativeCallBehavior::ChannelConstructor)
        );
        assert_eq!(
            spec("range").map(|native| native.behavior),
            Some(NativeCallBehavior::RangeConstructor)
        );
        assert_eq!(
            spec("println").map(|native| native.behavior),
            Some(NativeCallBehavior::Standard)
        );
    }

    #[test]
    fn variable_native_arities_are_explicit() {
        assert_eq!(native_arity("range"), Some(Arity::Range { min: 1, max: 3 }));
        assert_eq!(native_arity("input"), Some(Arity::Range { min: 0, max: 1 }));
        assert_eq!(native_arity("exit"), Some(Arity::Range { min: 0, max: 1 }));
        assert_eq!(native_arity("print"), Some(Arity::AtLeast(1)));
        assert_eq!(native_arity("println"), Some(Arity::AtLeast(1)));
        assert_eq!(native_arity("format"), Some(Arity::AtLeast(1)));
        assert_eq!(native_arity("Set"), Some(Arity::AtLeast(0)));
        assert_eq!(native_arity("spawn"), Some(Arity::AtLeast(1)));
        assert_eq!(native_arity("select"), Some(Arity::Range { min: 1, max: 2 }));
        assert_eq!(native_arity("channel"), Some(Arity::Range { min: 0, max: 1 }));

        assert!(!Arity::Range { min: 1, max: 3 }.accepts(0));
        assert!(Arity::Range { min: 1, max: 3 }.accepts(2));
        assert!(!Arity::Range { min: 1, max: 3 }.accepts(4));
    }

    #[test]
    fn every_runtime_native_has_one_contract() {
        let specs = registry();
        let mut names = std::collections::HashSet::new();

        for spec in specs.iter().filter(|spec| matches!(spec.kind, NativeKind::Runtime)) {
            assert!(names.insert(spec.name), "native dupliquée: {}", spec.name);
        }

        assert_eq!(names.len(), 77);
        assert!(names.contains("process_run"));
        assert!(!names.contains(Intrinsic::Spawn.name()));
        assert!(!names.contains(Intrinsic::Yield.name()));
        assert!(!names.contains(Intrinsic::Sleep.name()));
        assert!(!names.contains(Intrinsic::Select.name()));
    }

    #[test]
    fn process_run_has_a_structural_record_result() {
        let ty = all().remove("process_run").expect("process_run doit exister");

        assert_eq!(
            ty,
            Type::Function(FunctionType {
                generic_params: Vec::new(),
                is_async: false,
                generic_constraints: Vec::new(),
                params: vec![Type::Str, Type::Dynamic],
                return_type: Box::new(Type::Record(vec![
                    ("stdout".into(), Type::Str),
                    ("stderr".into(), Type::Str),
                    ("code".into(), Type::Int),
                    ("success".into(), Type::Bool),
                ])),
            })
        );
    }
}
