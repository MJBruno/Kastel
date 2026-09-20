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

