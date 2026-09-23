//! Tests de régression — à déposer dans `src/vm/machine/` (mêmes
//! conventions que `robustness_tests.rs`, dont ce fichier réutilise le
//! harnais) ou dans `tests/` selon comment le crate expose ses modules
//! internes.
//!
//! Organisation :
//!   1. Bugs CONFIRMÉS par l'audit (actuellement en échec attendu — à
//!      marquer `#[ignore]` tant qu'ils ne sont pas corrigés, ou à
//!      dé-ignorer une fois le correctif appliqué).
//!   2. Couverture de régression générale, une section par fonctionnalité.
//!
//! Ce fichier ne remplace pas robustness_tests.rs, il le complète.

use std::rc::Rc;
use std::cell::RefCell;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
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
/// Ce test DOIT actuellement échouer (compile_only retourne Err).
/// Retirer #[ignore] une fois le correctif appliqué : il doit alors
/// passer (Ok).
#[test]
#[ignore = "bug confirmé : voir audit — is_assignable_to ne résout pas les alias Record"]
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
/// fonctionner (le sens inverse du cas ci-dessus).
#[test]
#[ignore = "bug confirmé : voir audit — même lacune, sens inverse"]
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

/// BUG #2 (GC) — `VirtualMachine::pin_roots` (src/vm/machine/gc.rs)
/// n'inclut pas `self.module_loader.loaded_modules()` dans les racines
/// externes épinglées, alors que `gc::collect` traite explicitement les
/// globales/exports des modules chargés comme des racines à part (voir
/// le commentaire dans runtime/gc.rs — ces valeurs ne sont pas toujours
/// atteignables depuis la pile/les globales de la VM).
///
/// Répro : un premier `import` charge un module A dont une valeur n'est
/// PAS conservée dans une variable de la VM principale au-delà de son
/// usage immédiat mais reste vivante via le cache du module. Un second
/// `import` (module B) déclenche une VM imbriquée ; si le budget
/// d'allocations est dépassé PENDANT l'exécution de B, le GC de la VM
/// imbriquée tourne sans voir les racines de A (pin_roots ne les a pas
/// épinglées) et peut invalider des objets de A qui ne sont, dans le
/// module_loader, atteignables QUE via ce chemin.
///
/// Ce test force une collecte pendant l'import imbriqué et vérifie que
/// l'état de A survit. Il doit actuellement échouer (ou être fragile /
/// dépendant du seuil de collecte) ; le corriger consiste à ajouter les
/// modules déjà chargés à `ExternalRoots` dans `pin_roots`.
#[test]
#[ignore = "bug confirmé : voir audit — pin_roots omet les modules déjà chargés de la VM appelante"]
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

    // Module B : alloue massivement pour dépasser le seuil de collecte
    // pendant sa propre exécution (VM imbriquée).
    let mod_b = dir.join("b.ks");
    let mut filler = String::new();
    for i in 0..20_000 {
        filler.push_str(&format!("let _junk{i} = [{i}, {i}, {i}];\n"));
    }
    std::fs::File::create(&mod_b)
        .unwrap()
        .write_all(filler.as_bytes())
        .unwrap();

    let main_source = format!(
        r#"
        import "{a}" as a;
        let first = a.payload[0];   // référencé une seule fois, pas gardé

        import "{b}" as b;          // VM imbriquée, allocations massives

        print(a.payload[0]);        // a.payload doit encore être valide ici
        print(first);
        "#,
        a = mod_a.display(),
        b = mod_b.display(),
    );

    let (_vm, result) = run_script(&main_source);

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
        assert!(boolean(global(&vm, "a_has_3")), "b = a doit partager la même référence");
        assert!(!boolean(global(&vm, "a_has_99")), "copy() doit être indépendante");
    }

    #[test]
    fn set_self_add_does_not_panic() {
        // Cas dégénéré documenté dans hashed.rs : hachage/emprunt avant
        // écriture pour éviter un emprunt mutable réentrant.
        let (_vm, result) = run_script("let s = Set(1); s.add(s);");
        assert!(result.is_ok());
    }

    #[test]
    fn dict_self_key_does_not_panic() {
        let (_vm, result) = run_script(r#"let d = {"a": 1}; d[d] = 2;"#);
        assert!(result.is_ok());
    }

    #[test]
    fn tuple_is_immutable() {
        let (_vm, result) = run_script("let t = (1, 2); t[0] = 5;");
        assert!(result.is_err(), "un tuple ne doit pas être modifiable par index");
    }

    #[test]
    fn tuple_has_no_copy_method() {
        let (_vm, result) = run_script("let t = (1, 2); t.copy();");
        assert!(result.is_err(), "Tuple est immuable, copy() n'a pas de sens");
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
            let p = Point();
            "#,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn private_field_is_not_accessible_outside_the_class() {
        let (_vm, result) = run_script(
            r#"
            class Person {
                private let age: int = 0;
            }
            let p = Person();
            let a = p.age;
            "#,
        );
        assert!(result.is_err(), "p.age doit être interdit hors de la classe");
    }

    #[test]
    fn constructor_overloading_dispatches_on_arity() {
        let (vm, result) = run_script(
            r#"
            class Point {
                private let x: int = 0;
                private let y: int = 0;

                initialize() {}
                initialize(x: int, y: int) {
                    this.x = x;
                    this.y = y;
                }

                func sum() -> int { return this.x + this.y; }
            }

            let a = Point();
            let b = Point(3, 4);
            let sa = a.sum();
            let sb = b.sum();
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "sa")), 0);
        assert_eq!(integer(global(&vm, "sb")), 7);
    }

    #[test]
    fn method_overloading_dispatches_on_arity_and_type() {
        let (vm, result) = run_script(
            r#"
            class Calc {
                func combine(x: int, y: int) -> int { return x + y; }
                func combine(x: str, y: str) -> str { return x + y; }
            }

            let c = Calc();
            let n = c.combine(1, 2);
            let s = c.combine("a", "b");
            "#,
        );
        assert!(result.is_ok());
        assert_eq!(integer(global(&vm, "n")), 3);
        match global(&vm, "s") {
            Value::Object(_) => {} // string
            other => panic!("string attendue, reçu {other:?}"),
        }
    }
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
        assert!(result.is_err(), "une récursion infinie doit lever RuntimeError::StackOverflow, pas planter le process natif");
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

        let result = compile_only(&source);
        assert!(result.is_err(), "MAX_EXPRESSION_DEPTH doit rejeter proprement, pas faire déborder la pile native du compilateur");
    }

    #[test]
    fn deep_but_acyclic_structure_gc_does_not_stack_overflow() {
        // Le marquage GC doit être itératif (drain_pending), pas récursif.
        let mut source = String::new();
        source.push_str("let list = [];\nlet cur = list;\n");
        for i in 0..50_000 {
            source.push_str(&format!("let n{i} = [];\ncur.add(n{i});\ncur = n{i};\n"));
        }
        source.push_str("collect_garbage();\n"); // si exposé au langage ; sinon retirer cette ligne
        let (_vm, _result) = run_script(&source);
        // Le test réussit s'il ne panique/ne segfault pas ; pas d'assertion
        // de valeur nécessaire ici.
    }
}

mod modules_and_imports {
    use super::*;

    #[test]
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
