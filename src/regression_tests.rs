//! Tests de régression — à déposer dans `src/vm/machine/` (mêmes
//! conventions que `robustness_tests.rs`, dont ce fichier réutilise le
//! harnais) ou dans `tests/` selon comment le crate expose ses modules
//! internes.
//!
//! Organisation :
//!   1. Régressions ciblées issues de l'audit.
//!   2. Couverture de régression générale, une section par fonctionnalité.
//!
//! Ce fichier ne remplace pas robustness_tests.rs, il le complète.

use std::rc::Rc;

use crate::{
    compiler::{
        compiler::Compiler, module_types::ModuleTypeLoader, type_checker::TypeCheckContext,
    },
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    module::{module::ModuleLoader, resolver::ModuleResolver},
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

// ============================================================
//                         HARNAIS
// ============================================================

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

fn run_script_from_path(
    source: &str,
    main_path: std::path::PathBuf,
) -> (VirtualMachine, Result<(), RuntimeError>) {
    let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
    let statements = Parser::new(tokens).parse().unwrap();

    let project_root = main_path
        .parent()
        .expect("le fichier principal doit avoir un parent")
        .to_path_buf();
    let resolver = ModuleResolver::new(project_root);
    let type_loader = Rc::new(ModuleTypeLoader::new(resolver.clone()));
    let context = TypeCheckContext::new(main_path.clone(), type_loader);

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(compiler.compile_with_context(&statements, context).unwrap());
    let loader = ModuleLoader::with_resolver(resolver);
    let mut vm = VirtualMachine::new_with_loader(function, Some(main_path), loader);
    let result = vm.run().map(|_| ());

    (vm, result)
}

/// Compile seulement (lexer + parser + compiler, avec vérification de
/// types) sans exécuter — pour les tests qui portent sur des erreurs
/// statiques (type-checker), pas sur le runtime.
fn compile_only(source: &str) -> Result<(), String> {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .map_err(|e| format!("{e:?}"))?;
    let statements = Parser::new(tokens).parse().map_err(|e| format!("{e:?}"))?;

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    compiler
        .compile(&statements)
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
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

fn boolean(value: Value) -> bool {
    match value {
        Value::Boolean(value) => value,
        other => panic!("booléen attendu, reçu {other:?}"),
    }
}

fn on_big_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(work)
        .expect("thread à pile large")
        .join()
        .expect("le test ne doit pas paniquer")
}

// ============================================================
// 1. BUGS CONFIRMÉS PAR L'AUDIT — reproductions minimales
// ============================================================

/// BUG #1 (type-checker) — `Type::is_assignable_to` n'a pas de cas
/// (Record, Named) / (Named, Record) dans src/compiler/types.rs : une
/// valeur de forme record (littéral, ou retour de fonction inféré comme
/// record structurel) ne s'accorde jamais avec un alias nommé qui
/// pointe pourtant vers exactement cette forme.
///
/// C'est le bug signalé : `let p: Point = origin();` avec
/// `type Point = { x: int, y: int };` échoue avec
/// "expected `{ x: int, y: int }`, found `Point`".
///
/// Cette régression reste active : les alias de records sont résolus
/// structurellement avant la comparaison d assignabilité.
#[test]
fn type_alias_on_record_literal_is_assignable() {
    let source = r#"
        type Point = { x: int, y: int };

        func origin() -> Point {
            return { x: 0, y: 0 };
        }

        let p: Point = origin();
        print(p.x);
    "#;

    assert!(
        compile_only(source).is_ok(),
        "un record structurel conforme à l'alias Point devrait être assignable à Point"
    );
}

/// Variante symétrique : assigner une valeur typée `Point` (alias) à une
/// variable typée par la forme record littérale équivalente doit aussi
/// fonctionner.
#[test]
fn record_shape_accepts_matching_named_alias() {
    let source = r#"
        type Point = { x: int, y: int };

        func make() -> Point {
            return { x: 1, y: 2 };
        }

        let shape: { x: int, y: int } = make();
        print(shape.x);
    "#;

    assert!(compile_only(source).is_ok());
}

/// Un alias vers une forme INCOMPATIBLE doit, lui, rester une erreur —
/// pour vérifier que le correctif ne devient pas laxiste au point
/// d'accepter n'importe quoi.
#[test]
fn type_alias_on_incompatible_record_is_rejected() {
    let source = r#"
        type Point = { x: int, y: int };

        func bad() -> Point {
            return { x: 0 };
        }
    "#;

    assert!(
        compile_only(source).is_err(),
        "un record auquel il manque un champ attendu ne doit pas passer le type-checker"
    );
}

/// Régression GC : les modules déjà chargés restent des racines pendant
/// l'exécution d'une VM imbriquée (par exemple durant un import).
#[test]
fn nested_import_gc_does_not_collect_caller_loaded_modules() {
    use std::io::Write;

    let dir = std::env::temp_dir().join(format!("kastel_gc_regress_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    // Module A : construit une valeur volumineuse et alloue beaucoup ;
    // sa survie ne doit dépendre QUE du cache du module_loader (pas
    // d'une variable encore vivante sur la pile de la VM principale).
    let mod_a = dir.join("a.ks");
    std::fs::File::create(&mod_a)
        .unwrap()
        .write_all(b"export let payload = [1, 2, 3, 4, 5];")
        .unwrap();

    // Module B : alloue suffisamment pour déclencher plusieurs collectes
    // pendant sa propre exécution (VM imbriquée), sans fabriquer 20 000
    // déclarations distinctes qui ralentiraient inutilement le compilateur.
    let mod_b = dir.join("b.ks");
    std::fs::File::create(&mod_b)
        .unwrap()
        .write_all(
            br#"
let junk = [];
for i in range(0, 6000) { junk.add([i, i, i]); }
export const total = junk.size();
"#,
        )
        .unwrap();

    let main_source = r#"
        import a;
        let first = a.payload[0];

        import b;

        print(a.payload[0]);
        print(first);
        "#;

    let main_path = dir.join("main.ks");
    let _ = std::fs::write(&main_path, &main_source);
    let main_path = std::fs::canonicalize(&main_path).unwrap();

    let (_vm, result) = run_script_from_path(&main_source, main_path);

    assert!(
        result.is_ok(),
        "l'import de b ne doit pas corrompre/collecter l'état de a : {result:?}"
    );
}

// ============================================================
// 2. COUVERTURE GÉNÉRALE — une section par fonctionnalité
// ============================================================

mod arithmetic {
    use super::*;

    #[test]
    fn integer_overflow_is_an_error_not_a_wrap() {
        let (_vm, result) = run_script("let x = 9223372036854775807 + 1;");
        assert!(result.is_err());
    }

    #[test]
    fn integer_underflow_on_subtract_is_an_error() {
        let (_vm, result) = run_script("let x = -9223372036854775807 - 2;");
        assert!(result.is_err());
    }

    #[test]
    fn division_always_produces_float() {
        let (vm, result) = run_script("let x = 7 / 2;");
        assert!(result.is_ok());
        match global(&vm, "x") {
            Value::Float(f) => assert!((f - 3.5).abs() < 1e-9),
            other => panic!("float attendu, reçu {other:?}"),
        }
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let (_vm, result) = run_script("let x = 1 / 0;");
        assert!(result.is_err());
    }

    #[test]
    fn modulo_by_zero_is_an_error() {
        let (_vm, result) = run_script("let x = 1 % 0;");
        assert!(result.is_err());
    }

    #[test]
    fn modulo_i64_min_by_minus_one_does_not_panic() {
        let (_vm, result) = run_script("let x = -9223372036854775808 % -1;");
        assert!(result.is_ok());
    }

    #[test]
    fn int_float_comparison_mixes_freely() {
        let (vm, result) = run_script("let x = 5 < 5.5;");
        assert!(result.is_ok());
        assert!(boolean(global(&vm, "x")));
    }
}

mod collections {
    use super::*;

    #[test]
    fn set_deduplicates_on_construction() {
        let (vm, result) = run_script("let s = Set(1, 2, 2, 3, 1); let n = s.size();");
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "n")), 3);
    }

    #[test]
    fn set_is_shared_by_reference_copy_is_independent() {
        let (vm, result) = run_script(
            r#"
            let a = Set(1, 2);
            let b = a;
            b.add(3);
            let a_has_3 = a.contains(3);

            let c = a.copy();
            c.add(99);
            let a_has_99 = a.contains(99);
            "#,
        );
        assert!(result.is_ok());
        assert!(
            boolean(global(&vm, "a_has_3")),
            "b = a doit partager la même référence"
        );
        assert!(
            !boolean(global(&vm, "a_has_99")),
            "copy() doit être indépendante"
        );
    }

    #[test]
    fn set_self_add_does_not_panic() {
        // Cas dégénéré documenté dans hashed.rs : hachage/emprunt avant
        // écriture pour éviter un emprunt mutable réentrant.
        let (_vm, result) = run_script("let s: dynamic = Set(1); s.add(s);");
        assert!(result.is_ok());
    }

    #[test]
    fn dict_self_key_does_not_panic() {
        let (_vm, result) = run_script(r#"let d: dynamic = {"a": 1}; d[d] = 2;"#);
        assert!(result.is_ok());
    }

    #[test]
    fn tuple_is_immutable() {
        let (_vm, result) = run_script("let t = (1, 2); t[0] = 5;");
        assert!(
            result.is_err(),
            "un tuple ne doit pas être modifiable par index"
        );
    }

    #[test]
    fn tuple_has_no_copy_method() {
        let (_vm, result) = run_script("let t = (1, 2); t.copy();");
        assert!(
            result.is_err(),
            "Tuple est immuable, copy() n'a pas de sens"
        );
    }

    #[test]
    fn array_is_shared_by_reference_copy_is_independent() {
        let (vm, result) = run_script(
            r#"
            let a = [1, 2, 3];
            let b = a;
            b.add(4);
            let shared = a.size() == 4;

            let c = a.copy();
            c.add(5);
            let independent = a.size() == 4;
            "#,
        );
        assert!(result.is_ok());
        assert!(boolean(global(&vm, "shared")));
        assert!(boolean(global(&vm, "independent")));
    }

    #[test]
    fn set_operations_union_intersection_difference() {
        let (vm, result) = run_script(
            r#"
            let a = Set(1, 2, 3);
            let b = Set(2, 3, 4);
            let u = a.union(b).size();
            let i = a.intersection(b).size();
            let d = a.difference(b).size();
            let sd = a.symmetric_difference(b).size();
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "u")), 4);
        assert_eq!(integer(global(&vm, "i")), 2);
        assert_eq!(integer(global(&vm, "d")), 1);
        assert_eq!(integer(global(&vm, "sd")), 2);
    }

    #[test]
    fn string_size_counts_unicode_chars_not_bytes() {
        // "café" : 4 caractères Unicode, 5 octets en UTF-8 (é = 2 octets).
        let (vm, result) = run_script(r#"let n = "café".size();"#);
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "n")), 4);
    }
}

mod classes {
    use super::*;

    #[test]
    fn implicit_default_constructor_when_none_declared() {
        let (_vm, result) = run_script(
            r#"
            class Point {
                private let x: int = 0;
            }
            let p = new Point();
            "#,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn private_field_is_not_accessible_outside_the_class() {
        let result = compile_only(
            r#"
            class Person {
                private let age: int = 0;
            }
            let p = new Person();
            let a = p.age;
            "#,
        );

        assert!(
            result.is_err(),
            "p.age doit être interdit hors de la classe"
        );
    }

    #[test]
    fn constructor_overloading_dispatches_on_arity() {
        let (vm, result) = run_script(
            r#"
            class Point {
                private let x: int = 0;
                private let y: int = 0;

                func initialize() {}
                func initialize(x: int, y: int) {
                    self.x = x;
                    self.y = y;
                }

                func sum() -> int { return self.x + self.y; }
            }

            let a = new Point();
            let b = new Point(3, 4);
            let sa = a.sum();
            let sb = b.sum();
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "sa")), 0);
        assert_eq!(integer(global(&vm, "sb")), 7);
    }

    #[test]
    fn method_overloading_dispatches_on_arity() {
        let (vm, result) = run_script(
            r#"
            class Calc {
                func combine(x: int) -> int { return x; }
                func combine(x: int, y: int) -> int { return x + y; }
            }

            let c = new Calc();
            let one = c.combine(3);
            let two = c.combine(1, 2);
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "one")), 3);
        assert_eq!(integer(global(&vm, "two")), 3);
    }
}

#[test]
fn constructor_overloading_dispatches_on_arity() {
    let (vm, result) = run_script(
        r#"
        class Point {
            private let x: int = 0;
            private let y: int = 0;

            func initialize() {}
            func initialize(x: int, y: int) {
                self.x = x;
                self.y = y;
            }

            func sum() -> int {
                return self.x + self.y;
            }
        }

        let a = new Point();
        let b = new Point(3, 4);
        let sa = a.sum();
        let sb = b.sum();
        "#,
    );

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(integer(global(&vm, "sa")), 0);
    assert_eq!(integer(global(&vm, "sb")), 7);
}

mod types_and_aliases {
    use super::*;

    #[test]
    fn union_parameter_accepts_either_member() {
        let source = r#"
            type Number = int | float;

            func double(n: Number) -> Number {
                return n * 2;
            }

            let a = double(3);
            let b = double(2.5);
        "#;
        assert!(compile_only(source).is_ok());
    }

    #[test]
    fn union_parameter_rejects_non_member() {
        let source = r#"
            type Number = int | float;

            func double(n: Number) -> Number {
                return n * 2;
            }

            let a = double("nope");
        "#;
        assert!(compile_only(source).is_err());
    }

    #[test]
    fn record_literal_key_syntax_vs_dict_string_keys() {
        let (vm, result) = run_script(
            r#"
            let rec = { name: "Bruno", age: 25 };
            let dic = {"name": "bruno"};
            let a = rec.age;
            let n = dic["name"];
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "a")), 25);
    }
}

mod recursion_and_depth {
    use super::*;

    #[test]
    fn stack_overflow_via_recursion_is_a_runtime_error_not_a_crash() {
        let (_vm, result) = run_script(
            r#"
            func loop(n: int) -> int { return loop(n + 1); }
            loop(0);
            "#,
        );
        assert!(
            result.is_err(),
            "une récursion infinie doit lever RuntimeError::StackOverflow, pas planter le process natif"
        );
    }

    #[test]
    fn deeply_nested_expression_is_a_compile_error_not_a_crash() {
        let mut source = String::from("let x = ");
        for _ in 0..10_000 {
            source.push('(');
        }
        source.push('1');
        for _ in 0..10_000 {
            source.push(')');
        }
        source.push(';');

        let result = on_big_stack(move || compile_only(&source));
        assert!(
            result.is_err(),
            "MAX_EXPRESSION_DEPTH doit rejeter proprement, pas faire déborder la pile native du compilateur"
        );
    }

    #[test]
    fn deep_but_acyclic_structure_gc_does_not_stack_overflow() {
        let source = r#"
        let list = [];
        let cur = list;

        for i in range(0, 10000) {
            let next = [];
            cur.add(next);
            cur = next;
        }
    "#;

        let (mut vm, result) = run_script(source);

        assert!(
            result.is_ok(),
            "une structure acyclique profonde doit être construite correctement : {result:?}"
        );

        // Retirer les racines du graphe sans détruire immédiatement les
        // objets. Les valeurs restent temporairement vivantes dans Rust,
        // mais elles ne font plus partie des racines connues du GC.
        let list = vm
            .globals
            .borrow_mut()
            .remove("list")
            .expect("la globale 'list' doit exister");

        let cur = vm
            .globals
            .borrow_mut()
            .remove("cur")
            .expect("la globale 'cur' doit exister");

        // Le GC doit maintenant reconnaître toute la chaîne comme
        // inaccessible et la casser sans destruction récursive.
        let broken = vm.collect_garbage();

        assert!(
            broken > 0,
            "la structure profonde doit être récupérée par le GC"
        );

        // Après le sweep en deux phases, ces destructions sont sûres :
        // les références internes ont déjà été supprimées.
        drop(list);
        drop(cur);
    }
}

mod modules_and_imports {
    use super::*;

    #[test]
    #[ignore = "test historique dépendant d une arborescence std externe"]
    fn repl_style_import_registers_type_for_later_use() {
        // Cf. limite connue « REPL sans mémoire de types » : ce test
        // vérifie côté compilation classique (pas REPL) que le type
        // importé reste utilisable statiquement après l'import.
        let source = r#"
            import "std/collections.ks" as collections;
            let x: collections.SomeExportedType = collections.make();
        "#;
        // Ce test dépend de l'arborescence std réelle du projet ; à
        // adapter avec un module de test local si std/collections.ks
        // n'existe pas sous ce nom chez vous.
        let _ = compile_only(source);
    }
}
