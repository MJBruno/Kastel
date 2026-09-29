use std::rc::Rc;

use crate::{
    compiler::compiler::Compiler,
    error::runtime_error::RuntimeError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

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

fn boolean(vm: &VirtualMachine, name: &str) -> bool {
    match global(vm, name) {
        Value::Boolean(value) => value,
        other => panic!("{name}: booléen attendu, reçu {other:?}"),
    }
}

#[test]
fn typed_catch_catches_runtime_errors_as_err() {
    let (vm, result) = run_script(
        r#"
        let caught = false;
        let kind = "";
        let message = "";
        let kind_method = "";
        let message_method = "";
        let rendered = "";

        try {
            let x = 10 / 0;
        } catch (e: Err) {
            caught = true;
            kind = e.kind;
            message = e.message;
            kind_method = e.kind();
            message_method = e.message();
            rendered = e.to_string();
        }
        "#,
    );

    result.expect("catch(e: Err) doit intercepter une erreur runtime");
    assert!(boolean(&vm, "caught"));
    assert_eq!(
        global(&vm, "kind").as_string_value().as_deref(),
        Some("DivisionByZero")
    );
    assert_eq!(
        global(&vm, "message").as_string_value().as_deref(),
        Some("Division by zero.")
    );
    assert_eq!(
        global(&vm, "kind_method").as_string_value().as_deref(),
        Some("DivisionByZero")
    );
    assert_eq!(
        global(&vm, "message_method").as_string_value().as_deref(),
        Some("Division by zero.")
    );
    assert_eq!(
        global(&vm, "rendered").as_string_value().as_deref(),
        Some("Err<DivisionByZero>(Division by zero.)")
    );
}

#[test]
fn typed_catch_does_not_swallow_other_thrown_values() {
    let (vm, result) = run_script(
        r#"
        let typed = false;
        let generic = false;
        let value = "";

        try {
            try {
                throw "boom";
            } catch (e: Err) {
                typed = true;
            }
        } catch (e) {
            generic = true;
            value = e;
        }
        "#,
    );

    result.expect("une exception non typée doit remonter au catch général");
    assert!(!boolean(&vm, "typed"));
    assert!(boolean(&vm, "generic"));
    assert_eq!(
        global(&vm, "value").as_string_value().as_deref(),
        Some("boom")
    );
}

#[test]
fn typed_catch_finally_runs_when_type_does_not_match() {
    let (vm, result) = run_script(
        r#"
        let finally_ran = false;
        let generic = false;

        try {
            try {
                throw "boom";
            } catch (e: Err) {
                let never = true;
            } finally {
                finally_ran = true;
            }
        } catch (e) {
            generic = true;
        }
        "#,
    );

    result.expect("finally doit s'exécuter avant la propagation");
    assert!(boolean(&vm, "finally_ran"));
    assert!(boolean(&vm, "generic"));
}

#[test]
fn result_err_is_not_a_runtime_exception_by_itself() {
    let (vm, result) = run_script(
        r#"
        let caught = false;
        let result = Err("bad");

        try {
            let value = result;
        } catch (e: Err) {
            caught = true;
        }
        "#,
    );

    result.expect("construire Result.Err ne doit pas lever d'exception");
    assert!(!boolean(&vm, "caught"));
}

// ============================================================
// STABILISATION try / catch / finally
// ============================================================

fn integer(vm: &VirtualMachine, name: &str) -> i64 {
    match global(vm, name) {
        Value::Integer(value) => value,
        other => panic!("{name}: entier attendu, reçu {other:?}"),
    }
}

#[test]
fn handler_is_removed_after_a_catch_completes() {
    // Avant : le handler restait empilé après le `catch`, et le `throw "b"`
    // suivant était « intercepté » par ce handler périmé, qui fermait le
    // frame au lieu de rejoindre le `catch` externe.
    let (vm, result) = run_script(
        r#"
        let outer = false;
        let inner = 0;

        try {
            try {
                throw "a";
            } catch (e) {
                inner = inner + 1;
            }
            throw "b";
        } catch (e2) {
            outer = true;
        }
        "#,
    );

    result.expect("le second throw doit atteindre le catch externe");
    assert!(boolean(&vm, "outer"));
    assert_eq!(integer(&vm, "inner"), 1);
}

#[test]
fn finally_runs_exactly_once_after_a_catch() {
    // Avant : le handler périmé (catch consommé, finally conservé) rejouait
    // le `finally` sur l'exception suivante.
    let (vm, result) = run_script(
        r#"
        let count = 0;

        try {
            try {
                throw "x";
            } catch (e) {
            } finally {
                count = count + 1;
            }
            throw "y";
        } catch (e) {
        }
        "#,
    );

    result.expect("aucune exception ne doit s'échapper");
    assert_eq!(integer(&vm, "count"), 1);
}

#[test]
fn nested_finally_does_not_steal_the_pending_exception() {
    // Avant : `pending_exception` était un emplacement unique ; le `finally`
    // de `cleanup()` (terminé normalement) consommait l'exception en attente
    // du `finally` englobant et la relançait trop tôt.
    let (vm, result) = run_script(
        r#"
        let order = [];
        let caught = "";

        func cleanup() {
            try {
                order.add("c");
            } finally {
                order.add("cf");
            }
        }

        try {
            try {
                throw "boom";
            } finally {
                cleanup();
                order.add("after");
            }
        } catch (e) {
            caught = e;
        }

        let ok = order.size() == 3 && order[2] == "after" && caught == "boom";
        "#,
    );

    result.expect("l'exception doit survivre au finally imbriqué");
    assert!(boolean(&vm, "ok"));
}

#[test]
fn exception_thrown_in_finally_replaces_the_pending_one() {
    let (vm, result) = run_script(
        r#"
        let caught = "";

        try {
            try {
                throw "first";
            } finally {
                throw "second";
            }
        } catch (e) {
            caught = e;
        }
        "#,
    );

    result.expect("le throw du finally doit être intercepté");
    assert_eq!(
        global(&vm, "caught").as_string_value().as_deref(),
        Some("second")
    );
}

#[test]
fn break_and_continue_run_finally_and_pop_the_handler() {
    let (vm, result) = run_script(
        r#"
        let runs = 0;
        let i = 0;

        while i < 3 {
            i = i + 1;
            try {
                if i == 2 { continue; }
                if i == 3 { break; }
            } finally {
                runs = runs + 1;
            }
        }

        let leaked = false;

        try {
            let j = 0;
            while j < 2 {
                j = j + 1;
                try {
                    break;
                } catch (e) {
                    leaked = true;
                }
            }
            throw "later";
        } catch (e) {
        }

        let ok = runs == 3 && i == 3 && !leaked;
        "#,
    );

    result.expect("break/continue dans un try doivent rester cohérents");
    assert!(boolean(&vm, "ok"));
}

#[test]
fn return_inside_try_runs_finally_once_and_keeps_the_value() {
    let (vm, result) = run_script(
        r#"
        let log = [];

        func f() -> int {
            try {
                return 1;
            } finally {
                log.add("f");
            }
            return 0;
        }

        func g() -> int {
            try {
                return 5;
            } finally {
                let x = 10;
                log.add(x);
            }
            return 0;
        }

        let a = f();
        let b = g();
        let ok = a == 1 && b == 5 && log.size() == 2 && log[0] == "f" && log[1] == 10;
        "#,
    );

    result.expect("return dans un try doit exécuter le finally une seule fois");
    assert!(boolean(&vm, "ok"));
}

#[test]
fn return_inside_finally_overrides_and_compiles() {
    // Avant : `return` dans un `finally` ré-inlinait son propre `finally`
    // sans fin (récursion infinie du compilateur).
    let (vm, result) = run_script(
        r#"
        func h() -> int {
            try {
                return 1;
            } finally {
                return 2;
            }
        }

        let value = h();
        "#,
    );

    result.expect("return dans finally");
    assert_eq!(integer(&vm, "value"), 2);
}

#[test]
fn integer_overflow_in_a_hot_loop_is_catchable() {
    // Avant : les opcodes « chauds » (AddLocalConst...) renvoyaient leur
    // erreur avec `?` hors de la boucle d'exécution, sans passer par les
    // handlers : le `catch` ne voyait jamais le dépassement d'entier.
    let (vm, result) = run_script(
        r#"
        func overflow() -> bool {
            let i = 9223372036854775807;

            try {
                i = i + 1;
            } catch (e: Err) {
                return true;
            }

            return false;
        }

        let caught = overflow();
        "#,
    );

    result.expect("le dépassement doit être intercepté");
    assert!(boolean(&vm, "caught"));
}

#[test]
fn closure_captured_in_try_survives_a_caught_exception() {
    // Avant : la pile était tronquée sans fermer les upvalues des locales du
    // `try` ; la closure lisait ensuite un slot supprimé puis réutilisé.
    let (vm, result) = run_script(
        r#"
        let saved = [];

        func run() {
            try {
                let x = 41;

                func f() -> int {
                    return x + 1;
                }

                saved.add(f);
                throw "a";
            } catch (e) {
                let y = 1000;
                let z = 2000;
            }
        }

        run();
        let g = saved[0];
        let value = g();
        "#,
    );

    result.expect("la closure doit garder sa valeur capturée");
    assert_eq!(integer(&vm, "value"), 42);
}
