//! Tests de robustesse : racines du GC, profondeur de récursion, structures
//! cycliques, imbrication extrême.
//!
//! Les tests qui recurse profondément s'exécutent dans un thread à pile
//! large (comme l'interpréteur lui-même, voir `main.rs`) : la pile par défaut
//! des threads de test est trop petite.

use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use crate::{
    compiler::{
        compiler::Compiler, module_types::ModuleTypeLoader, type_checker::TypeCheckContext,
    },
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    module::{module::ModuleLoader, resolver::ModuleResolver},
    runtime::{gc, value::Value},
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

fn on_big_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(work)
        .expect("thread à pile large")
        .join()
        .expect("le test ne doit pas paniquer")
}

fn run_script(source: &str) -> (VirtualMachine, Result<(), RuntimeError>) {
    let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
    let statements = Parser::new(tokens).parse().unwrap();

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(compiler.compile(&statements).unwrap());

    let mut vm = VirtualMachine::new(function, None);
    let result = vm.run().map(|_| ());

    (vm, result)
}

fn global(vm: &VirtualMachine, name: &str) -> Value {
    vm.globals
        .borrow()
        .get(name)
        .cloned()
        .unwrap_or_else(|| panic!("globale '{name}' introuvable"))
}

fn integer(value: Value) -> i64 {
    match value {
        Value::Integer(value) => value,
        other => panic!("entier attendu, reçu {other:?}"),
    }
}

// ============================================================
//                      RACINES DU GC
// ============================================================

#[test]
fn map_results_survive_garbage_collection() {
    // Le rappel alloue un tableau par élément : le GC se déclenche pendant
    // `map`, alors que les résultats intermédiaires ne sont tenus que par une
    // variable Rust. Avant l'enracinement, ils étaient VIDÉS.
    let (size, ok) = on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
let a = [];
for i in range(0, 3000) { a.add(i); }
let pairs = a.map(func(x) { return [x, x * 2]; });
let ok = true;
for p in pairs {
    if p.size() != 2 { ok = false; }
}
let n = pairs.size();
"#,
        );

        result.unwrap();

        (
            integer(global(&vm, "n")),
            matches!(global(&vm, "ok"), Value::Boolean(true)),
        )
    });

    assert_eq!(size, 3000);
    assert!(ok, "des résultats de map ont été vidés par le GC");
}

#[test]
fn constructor_arguments_survive_field_initializers() {
    // `new Holder(...)` retire les arguments de la pile ; l'initialiseur du
    // champ `n` alloue beaucoup (donc déclenche le GC) AVANT le constructeur.
    let size = on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
func build() {
    let r = [];
    for i in range(0, 3000) { r.add([i]); }
    return r.size();
}

class Holder {
    public let n: int = build();

    func initialize(data) {
        this.data = data;
    }
}

let h = new Holder([[1, 2], [3, 4]]);
let a = h.data.get(0).size();
"#,
        );

        result.unwrap();

        integer(global(&vm, "a"))
    });

    assert_eq!(size, 2);
}

#[test]
fn import_does_not_corrupt_the_callers_values() {
    // `import` exécute le module dans une VM imbriquée. Ses collectes doivent
    // voir aussi `keep`, qui appartient au programme principal.
    let root = std::env::temp_dir().join(format!("kastel_gc_import_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    fs::write(
        root.join("heavy.ks"),
        r#"
let junk = [];
for i in range(0, 5000) { junk.add([i]); }
export const total = junk.size();
"#,
    )
    .unwrap();

    let main_source = r#"
let keep = [[1, 2], [3, 4]];
import heavy;
let a = keep.get(0).size();
let b = keep.get(1).get(0);
let t = heavy.total;
"#;

    let main_path = root.join("main.ks");
    fs::write(&main_path, main_source).unwrap();
    let main_path = fs::canonicalize(&main_path).unwrap();
    let project_root = root.clone();

    let values = on_big_stack(move || {
        let tokens = Lexer::new(main_source.to_string()).scan_token().unwrap();
        let statements = Parser::new(tokens).parse().unwrap();

        let resolver = ModuleResolver::new(project_root);
        let type_loader = Rc::new(ModuleTypeLoader::new(resolver.clone()));
        let context = TypeCheckContext::new(main_path.clone(), type_loader);

        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        let function = Rc::new(
            compiler
                .compile_with_context(&statements, context)
                .expect("compilation"),
        );

        let mut vm = VirtualMachine::new_with_loader(
            function,
            Some(main_path),
            ModuleLoader::with_resolver(resolver),
        );

        vm.run().expect("exécution");

        (
            integer(global(&vm, "a")),
            integer(global(&vm, "b")),
            integer(global(&vm, "t")),
        )
    });

    let _ = fs::remove_dir_all(&root);

    assert_eq!(values, (2, 3, 5000));
}

#[test]
fn marking_a_very_deep_structure_does_not_overflow_the_stack() {
    // 100 000 tableaux imbriqués : le marquage récursif d'origine faisait
    // déborder la pile ; il est désormais itératif.
    on_big_stack(|| {
        let mut value = Value::new_array(Vec::new());

        for _ in 0..100_000 {
            value = Value::new_array(vec![value]);
        }

        let stack = vec![value.clone()];
        let globals: HashMap<String, Value> = HashMap::new();

        gc::collect(gc::GcRoots {
            temp: &[],
            stack: &stack,
            globals: &globals,
            modules: &[],
            frames: &[],
            open_upvalues: &[],
            pending_exception: &None,
        });

        // Le destructeur d'Rc est récursif : on évite de le déclencher ici.
        std::mem::forget(stack);
        std::mem::forget(value);
    });
}

// ============================================================
//                       PROFONDEUR
// ============================================================

#[test]
fn infinite_recursion_is_reported_instead_of_exhausting_memory() {
    let outcome = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
func f(n) { return f(n + 1); }
f(0);
"#,
        );

        matches!(result, Err(RuntimeError::StackOverflow { .. }))
    });

    assert!(outcome);
}

#[test]
fn legitimate_deep_recursion_still_works() {
    let total = on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
func sum(n) {
    if n == 0 { return 0; }
    return n + sum(n - 1);
}
let total = sum(5000);
"#,
        );

        result.unwrap();

        integer(global(&vm, "total"))
    });

    assert_eq!(total, 12_502_500);
}

#[test]
fn nested_native_callbacks_are_bounded() {
    // Chaque niveau passe par `map` (rappel natif -> Rust récursif).
    let outcome = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
func g(n) {
    return [1].map(func(x) { return g(n + 1); });
}
let r = g(0);
"#,
        );

        matches!(result, Err(RuntimeError::StackOverflow { .. }))
    });

    assert!(outcome);
}

#[test]
fn absurdly_nested_source_is_rejected_by_the_parser() {
    let rejected = on_big_stack(|| {
        let source = format!("{}1{}", "(".repeat(2000), ")".repeat(2000));
        let tokens = Lexer::new(source).scan_token().unwrap();

        Parser::new(tokens).parse().is_err()
    });

    assert!(rejected);
}

#[test]
fn a_very_long_operator_chain_is_rejected_by_the_compiler() {
    // Le parser construit la chaîne itérativement, mais l'arbre a la
    // profondeur du nombre de termes : le compilateur (récursif) refuse.
    let rejected = on_big_stack(|| {
        let source = format!("let x = 1{};", " + 1".repeat(6000));
        let tokens = Lexer::new(source).scan_token().unwrap();
        let statements = Parser::new(tokens).parse().unwrap();

        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        compiler.compile(&statements).is_err()
    });

    assert!(rejected);
}

// ============================================================
//                   STRUCTURES CYCLIQUES
// ============================================================

#[test]
fn displaying_a_cyclic_array_terminates() {
    let shown = on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
let a = [];
a.add(a);
let s = str(a);
"#,
        );

        result.unwrap();

        global(&vm, "s").to_string()
    });

    assert_eq!(shown, "[...]");
}

#[test]
fn json_encode_of_a_cyclic_array_is_an_error_not_a_crash() {
    let outcome = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
let a = [];
a.add(a);
let j = json_encode(a);
"#,
        );

        matches!(result, Err(RuntimeError::CyclicStructure))
    });

    assert!(outcome);
}

// ============================================================
//                    ENTIERS 64 BITS
// ============================================================
//
// `+`, `-`, `*` et la négation lèvent `IntegerOverflow` : plus de
// « bouclage » silencieux (`factorial(21)` renvoyait un négatif).

fn overflows(source: &str) -> bool {
    let source = source.to_string();

    on_big_stack(move || {
        matches!(
            run_script(&source).1,
            Err(RuntimeError::IntegerOverflow { .. })
        )
    })
}

fn value_of(source: &str, name: &'static str) -> i64 {
    let source = source.to_string();

    on_big_stack(move || {
        let (vm, result) = run_script(&source);

        result.unwrap();

        integer(global(&vm, name))
    })
}

#[test]
fn addition_overflow_is_an_error_on_every_execution_path() {
    // Chemin générique (globales).
    assert!(overflows("let big = 9223372036854775807; let r = big + 1;"));

    // `x = x + constante` sur une locale (instruction fusionnée).
    assert!(overflows(
        "func f() { let x = 9223372036854775807; x = x + 1; return x; } f();"
    ));

    // `a = a + b` sur deux locales (instruction fusionnée).
    assert!(overflows(
        "func g() { let a = 9223372036854775807; let b = 1; a = a + b; return a; } g();"
    ));

    // Compteur de boucle.
    assert!(overflows(
        "func h() { let i = 9223372036854775805; while i < 9223372036854775807 { i = i + 5; } return i; } h();"
    ));
}

#[test]
fn subtraction_multiplication_and_negation_overflow_are_errors() {
    assert!(overflows("let s = -9223372036854775807 - 2;"));
    assert!(overflows("let m = 4611686018427387904 * 2;"));
    assert!(overflows(
        "let lowest = -9223372036854775808; let n = -lowest;"
    ));
}

#[test]
fn values_at_the_limits_are_still_valid() {
    // i64::MIN est un littéral valide ; les résultats qui tiennent passent.
    assert_eq!(
        value_of(
            "let lowest = -9223372036854775808; let r = lowest + 1;",
            "r"
        ),
        i64::MIN + 1
    );
    assert_eq!(
        value_of("let r = 9223372036854775807 - 1 + 1;", "r"),
        i64::MAX
    );

    // Un flottant dans l'opération : promotion en flottant, pas d'erreur.
    let promoted = on_big_stack(|| {
        let (vm, result) = run_script("let r = 9223372036854775807 + 1.0;");

        result.unwrap();

        matches!(global(&vm, "r"), Value::Float(_))
    });

    assert!(promoted);
}

#[test]
fn natives_report_overflow_instead_of_wrong_or_clamped_values() {
    assert!(overflows("let a = abs(-9223372036854775808);"));
    assert!(overflows("let p = pow(2, 63);"));
    assert!(overflows("let i = int(1e30);"));
    assert!(overflows("let f = floor(1e30);"));

    assert_eq!(value_of("let p = pow(2, 62);", "p"), 1i64 << 62);
    assert_eq!(value_of("let p = pow(-1, 5000000001);", "p"), -1);
    assert_eq!(value_of("let p = pow(1, 5000000001);", "p"), 1);
}

#[test]
fn factorial_is_exact_up_to_20_and_an_error_beyond() {
    let factorial = r#"
func fact(n) {
    let r = 1;
    let i = 2;

    while i <= n {
        r = r * i;
        i = i + 1;
    }

    return r;
}
"#;

    assert_eq!(
        value_of(&format!("{factorial}let r = fact(20);"), "r"),
        2_432_902_008_176_640_000
    );

    assert!(overflows(&format!("{factorial}let r = fact(21);")));
}

#[test]
fn wrapping_functions_are_the_explicit_cyclic_arithmetic() {
    assert_eq!(
        value_of("let w = wrapping_add(9223372036854775807, 1);", "w"),
        i64::MIN
    );
    assert_eq!(
        value_of("let w = wrapping_mul(4611686018427387904, 2);", "w"),
        i64::MIN
    );
    assert_eq!(
        value_of("let w = wrapping_sub(-9223372036854775808, 1);", "w"),
        i64::MAX
    );
}

#[test]
fn integer_division_is_exact_and_rounds_toward_negative_infinity() {
    use crate::stdlib::math::native_idiv;

    let idiv = |a: i64, b: i64| native_idiv(&[Value::Integer(a), Value::Integer(b)]);

    assert_eq!(integer(idiv(7, 2).unwrap()), 3);
    assert_eq!(integer(idiv(-7, 2).unwrap()), -4);
    assert_eq!(integer(idiv(7, -2).unwrap()), -4);
    assert_eq!(integer(idiv(-7, -2).unwrap()), 3);
    assert_eq!(integer(idiv(6, 3).unwrap()), 2);

    // Exact au-delà de 2^53, là où `floor(a / b)` (flottant) se trompe.
    assert_eq!(
        integer(idiv(9_007_199_254_740_993, 1).unwrap()),
        9_007_199_254_740_993
    );

    assert!(matches!(
        idiv(i64::MIN, -1),
        Err(RuntimeError::IntegerOverflow { .. })
    ));
    assert!(matches!(idiv(1, 0), Err(RuntimeError::DivisionByZero)));
}

#[test]
fn a_literal_that_does_not_fit_in_64_bits_is_explained() {
    let tokens = Lexer::new("let x = 9223372036854775808;".to_string())
        .scan_token()
        .unwrap();

    let errors = Parser::new(tokens).parse().unwrap_err();

    assert!(
        errors[0].message.contains("trop grand"),
        "message inattendu : {}",
        errors[0].message
    );
}

// ============================================================
//                 RECORDS, DICTS ET UNIONS
// ============================================================

fn text_of(source: &str, name: &'static str) -> String {
    let source = source.to_string();

    on_big_stack(move || {
        let (vm, result) = run_script(&source);

        result.unwrap();

        global(&vm, name).to_string()
    })
}

#[test]
fn record_fields_are_read_and_written_by_name() {
    let source = r#"
let p = { name: "Bruno", age: 25 };
p.age = p.age + 1;
let a = p.age;
let n = p.name;
"#;

    assert_eq!(value_of(source, "a"), 26);
    assert_eq!(text_of(source, "n"), "Bruno");
}

#[test]
fn record_is_shared_by_reference_and_copy_is_independent() {
    let source = r#"
let a = { x: 1 };
let b = a;
b.x = 5;
let shared = a.x;

let c = a.copy();
c.x = 9;
let original = a.x;
"#;

    assert_eq!(value_of(source, "shared"), 5);
    assert_eq!(value_of(source, "original"), 5);
}

#[test]
fn record_can_hold_functions_and_offers_introspection() {
    let greeting = text_of(
        r#"
let p = { name: "B", greet: func() { return "salut"; } };
let g = p.greet();
"#,
        "g",
    );

    assert_eq!(greeting, "salut");

    assert_eq!(
        value_of(
            r#"let p = { a: 1, b: 2, c: 3 }; let n = p.keys().size();"#,
            "n"
        ),
        3
    );
}

#[test]
fn record_shape_is_fixed_at_runtime() {
    // `r` est dynamique : le vérificateur ne peut pas refuser l'ajout, la
    // VM le fait.
    let refused = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
func add_field(r) { r.extra = 2; }
let p = { a: 1 };
add_field(p);
"#,
        );

        matches!(result, Err(RuntimeError::ObjectFieldNotFound { .. }))
    });

    assert!(refused);
}

#[test]
fn dict_is_read_by_key_not_by_dot_and_the_error_says_so() {
    let source = r#"
let d = {"name": "Bruno"};
let by_key = d["name"];
let by_get = d.get("name");
"#;

    assert_eq!(text_of(source, "by_key"), "Bruno");
    assert_eq!(text_of(source, "by_get"), "Bruno");

    let suggestion = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
let d = {"name": "Bruno"};
let x = d.name;
"#,
        );

        match result {
            Err(RuntimeError::ObjectFieldNotFound { suggestion, .. }) => suggestion,
            other => panic!("erreur guidée attendue, reçu {other:?}"),
        }
    });

    assert_eq!(suggestion.as_deref(), Some("[\"name\"]"));
}

#[test]
fn records_and_dicts_have_distinct_displays_and_type_names() {
    assert_eq!(
        text_of(r#"let s = str({ name: "Bruno", age: 25 });"#, "s"),
        "{name: \"Bruno\", age: 25}"
    );
    assert_eq!(
        text_of(r#"let s = str({"name": "Bruno"});"#, "s"),
        "{\"name\": \"Bruno\"}"
    );

    assert_eq!(text_of("let t = type({ a: 1 });", "t"), "record");
    assert_eq!(text_of(r#"let t = type({"a": 1});"#, "t"), "dict");
    assert_eq!(text_of("let t = type([1]);", "t"), "list");
}

#[test]
fn a_record_encodes_to_a_json_object() {
    assert_eq!(
        text_of(r#"let j = json_encode({ name: "B", age: 1 });"#, "j"),
        "{\"name\":\"B\",\"age\":1}"
    );
}

#[test]
fn union_typed_values_work_at_runtime() {
    let is_float = on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
type Number = int | float;

func half(x: Number) -> float {
    return x / 2;
}

let h = half(3);
"#,
        );

        result.unwrap();

        matches!(global(&vm, "h"), Value::Float(value) if value == 1.5)
    });

    assert!(is_float);
}

// ============================================================
//        SURCHARGE DE FONCTIONS, CLASSES IMPORTÉES, PRIVÉ
// ============================================================

/// Exécute `main_source` dans un projet temporaire contenant `files`.
/// `Err("compile: ...")` = refusé à la compilation (vérificateur ou
/// compilateur) ; `Err("runtime: ...")` = échec à l'exécution.
fn run_project<T: Send + 'static>(
    name: &str,
    files: &[(&str, &str)],
    main_source: &str,
    extract: impl FnOnce(&VirtualMachine) -> T + Send + 'static,
) -> Result<T, String> {
    let root = std::env::temp_dir().join(format!("kastel_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    for (file, content) in files {
        fs::write(root.join(file), content).unwrap();
    }

    let main_path = root.join("main.ks");
    fs::write(&main_path, main_source).unwrap();
    let main_path = fs::canonicalize(&main_path).unwrap();

    let project_root = root.clone();
    let main_source = main_source.to_string();

    let outcome = on_big_stack(move || {
        let tokens = Lexer::new(main_source)
            .scan_token()
            .map_err(|_| "compile: lexer")?;
        let statements = Parser::new(tokens)
            .parse()
            .map_err(|errors| format!("compile: parser {}", errors[0].message))?;

        let resolver = ModuleResolver::new(project_root);
        let type_loader = Rc::new(ModuleTypeLoader::new(resolver.clone()));
        let context = TypeCheckContext::new(main_path.clone(), type_loader);

        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        let function = Rc::new(
            compiler
                .compile_with_context(&statements, context)
                .map_err(|error| format!("compile: {error}"))?,
        );

        let mut vm = VirtualMachine::new_with_loader(
            function,
            Some(main_path),
            ModuleLoader::with_resolver(resolver),
        );

        vm.run().map_err(|error| format!("runtime: {error}"))?;

        Ok::<T, String>(extract(&vm))
    });

    let _ = fs::remove_dir_all(&root);

    outcome
}

#[test]
fn free_functions_are_overloaded_by_arity() {
    let source = r#"
func describe() { return "zéro"; }
func describe(a) { return "un"; }
func describe(a, b) { return "deux"; }

let r0 = describe();
let r1 = describe(1);
let r2 = describe(1, 2);

// Une fonction surchargée se passe comme une valeur.
let f = describe;
let r3 = f(1, 2);

// Et comme rappel : `map` appelle avec UN argument.
let mapped = [1, 2, 3].map(describe);
let first = mapped.get(0);
"#;

    assert_eq!(text_of(source, "r0"), "zéro");
    assert_eq!(text_of(source, "r1"), "un");
    assert_eq!(text_of(source, "r2"), "deux");
    assert_eq!(text_of(source, "r3"), "deux");
    assert_eq!(text_of(source, "first"), "un");
}

#[test]
fn overloaded_function_call_with_no_matching_arity_is_an_error() {
    // `g` est dynamique : le vérificateur ne peut pas refuser l'appel.
    let refused = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
func describe(a) { return "un"; }
func describe(a, b) { return "deux"; }

func call_with_three(g) { return g(1, 2, 3); }
call_with_three(describe);
"#,
        );

        matches!(result, Err(RuntimeError::WrongArgumentCount { .. }))
    });

    assert!(refused);
}

#[test]
fn exported_overloaded_functions_work_across_modules() {
    let shapes = r#"
export func area(side) { return side * side; }
export func area(width, height) { return width * height; }
"#;

    let main = r#"
import shapes;
from shapes import area;

let square = shapes.area(3);
let rectangle = shapes.area(2, 5);
let direct = area(4);
let direct2 = area(4, 6);
"#;

    let values = run_project("overload_modules", &[("shapes.ks", shapes)], main, |vm| {
        (
            integer(global(vm, "square")),
            integer(global(vm, "rectangle")),
            integer(global(vm, "direct")),
            integer(global(vm, "direct2")),
        )
    })
    .expect("le projet doit s'exécuter");

    assert_eq!(values, (9, 10, 16, 24));
}

const PERSONNE_MODULE: &str = r#"
export class Personne {
    private let age: int = 0;

    func initialize(name: str) {
        this.name = name;
    }

    func initialize(name: str, age: int) {
        this.name = name;
        this.age = age;
    }

    func getAge() -> int {
        return this.age;
    }
}
"#;

#[test]
fn imported_classes_are_checked_statically_like_local_ones() {
    let files = [("personne.ks", PERSONNE_MODULE)];

    // Constructeurs surchargés : 1 et 2 arguments valides.
    let ok = run_project(
        "imported_ok",
        &files,
        r#"
import personne.Personne;
let a = new Personne("A");
let b = new Personne("B", 30);
let age = b.getAge();
"#,
        |vm| integer(global(vm, "age")),
    );

    assert_eq!(ok, Ok(30));

    let compile_error = |main: &str| -> String {
        run_project("imported_err", &files, main, |_| ()).expect_err("doit être refusé")
    };

    // Aucune surcharge à 3 arguments : refusé à la COMPILATION.
    let arity = compile_error("import personne.Personne; let p = new Personne(\"A\", 1, 2);");
    assert!(arity.starts_with("compile:"), "{arity}");

    // Champ privé d'une classe importée : refusé à la COMPILATION.
    let private =
        compile_error("import personne.Personne; let p = new Personne(\"A\"); let x = p.age;");
    assert!(private.starts_with("compile:"), "{private}");
    assert!(private.contains("privé"), "{private}");

    // Alias d'import : `P` désigne `Personne`.
    let aliased = run_project(
        "imported_alias",
        &files,
        r#"
from personne import Personne as P;
let p: P = new P("A", 7);
let age = p.getAge();
"#,
        |vm| integer(global(vm, "age")),
    );

    assert_eq!(aliased, Ok(7));
}

#[test]
fn a_private_constructor_forbids_new_outside_the_class() {
    // Refusé à la compilation quand la classe est connue.
    let refused = on_big_stack(|| {
        let tokens = Lexer::new(
            r#"
class Solo {
    private func initialize() { this.v = 1; }
}
let s = new Solo();
"#
            .to_string(),
        )
        .scan_token()
        .unwrap();
        let statements = Parser::new(tokens).parse().unwrap();

        let mut compiler = Compiler::new();
        execute_native(&mut compiler);

        compiler.compile(&statements).is_err()
    });

    assert!(refused);

    // Une classe dérivée qui déclare son propre constructeur public peut
    // déléguer au constructeur privé de sa base (`base.initialize()`).
    let value = value_of(
        r#"
class Base {
    private func initialize() { this.v = 7; }
}

class Derived: Base {
    func initialize() { base.initialize(); }
}

let d = new Derived();
let v = d.v;
"#,
        "v",
    );

    assert_eq!(value, 7);
}

#[test]
fn json_and_display_never_expose_private_fields() {
    // `json_encode` n'encode pas les instances ; l'affichage n'en montre pas
    // les champs.
    assert_eq!(
        text_of(
            r#"
class Secret { private let key: int = 42; }
let s = new Secret();
let shown = str(s);
"#,
            "shown"
        ),
        "<Secret instance>"
    );

    let refused = on_big_stack(|| {
        let (_vm, result) = run_script(
            r#"
class Secret { private let key: int = 42; }
let s = new Secret();
let j = json_encode(s);
"#,
        );

        result.is_err()
    });

    assert!(refused);
}

// ============================================================
//           ALIAS DE TYPE EXPORTÉS ENTRE MODULES
// ============================================================

const SHAPES_MODULE: &str = r#"
export type Point = { x: int, y: int };
export type Number = int | float;

export func origin() -> Point {
    return { x: 0, y: 0 };
}
"#;

#[test]
fn importing_a_type_only_alias_does_not_break_at_runtime() {
    // Avant le correctif, importer un nom qui n'est QU'un alias de type
    // (aucune valeur exportée) faisait planter l'exécution : le bytecode
    // généré essayait de lire une propriété absente du module. Ici,
    // `from shapes import Point;` ne doit produire AUCUN bytecode pour
    // `Point`, et le reste de la ligne doit s'exécuter normalement.
    let files = [("shapes.ks", SHAPES_MODULE)];

    let value = run_project(
        "type_alias_from_import",
        &files,
        r#"
from shapes import Point, origin;

let p: Point = origin();
let x: int = p.x;
"#,
        |vm| integer(global(vm, "x")),
    );

    assert_eq!(value, Ok(0));

    // `import m.Point;` (forme à un seul nom) : même garantie.
    let value = run_project(
        "type_alias_import_dot",
        &files,
        r#"
import shapes.Point;

let p: Point = { x: 3, y: 4 };
let x: int = p.x;
"#,
        |vm| integer(global(vm, "x")),
    );

    assert_eq!(value, Ok(3));

    // `from m import *;` : les alias de type suivent aussi le `*`.
    let value = run_project(
        "type_alias_wildcard",
        &files,
        r#"
from shapes import *;

let n: Number = 5;
let p: Point = { x: 1, y: 2 };
let x: int = p.x + n;
"#,
        |vm| integer(global(vm, "x")),
    );

    assert_eq!(value, Ok(6));
}

#[test]
fn a_type_only_import_does_not_leak_a_runtime_binding() {
    // `Point` n'existe QUE comme type : y faire référence comme VALEUR à
    // l'exécution (et non dans une annotation de type) doit rester une
    // erreur — l'import n'a créé aucune globale "Point".
    let files = [("shapes.ks", SHAPES_MODULE)];

    let refused = run_project(
        "type_alias_no_runtime_leak",
        &files,
        r#"
from shapes import Point;
let x = Point;
"#,
        |_| (),
    );

    assert!(refused.is_err(), "{refused:?}");
}

#[test]
fn union_typed_parameters_work_across_modules_end_to_end() {
    // Couvre bout en bout (typage ET exécution) la régression de
    // `register_aliases` : `half` est déclarée AVANT l'alias `Number` dans
    // le fichier source, et l'appel se fait depuis un AUTRE module.
    let files = [(
        "mathx.ks",
        r#"
export func half(n: Number) -> float {
    return n / 2;
}

export type Number = int | float;
export type Classifier = int | str;

export func classify(x: Classifier) -> str {
    if x == 1 {
        return "one";
    }
    return "other";
}
"#,
    )];

    let value = run_project(
        "union_param_cross_module",
        &files,
        r#"
from mathx import half, classify;

let a = half(4);
let b = half(4.5);
let c = classify(1);
let d = classify("x");
let sum = a + b;
"#,
        |vm| {
            let sum = match global(vm, "sum") {
                Value::Float(value) => value,
                other => panic!("flottant attendu, reçu {other:?}"),
            };
            let c = global(vm, "c").to_string();
            let d = global(vm, "d").to_string();
            (sum, c, d)
        },
    )
    .expect("le projet doit s'exécuter");

    assert_eq!(value.0, 4.25);
    assert_eq!(value.1, "one");
    assert_eq!(value.2, "other");
}
