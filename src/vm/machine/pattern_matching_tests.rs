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

fn compile_only(source: &str) -> Result<(), String> {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .map_err(|error| format!("{error:?}"))?;
    let statements = Parser::new(tokens)
        .parse()
        .map_err(|error| format!("{error:?}"))?;

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    compiler
        .compile(&statements)
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}

fn global(vm: &VirtualMachine, name: &str) -> Value {
    vm.globals
        .borrow()
        .get(name)
        .cloned()
        .unwrap_or_else(|| panic!("globale '{name}' introuvable"))
}

fn integer(vm: &VirtualMachine, name: &str) -> i64 {
    match global(vm, name) {
        Value::Integer(value) => value,
        other => panic!("{name}: entier attendu, reçu {other:?}"),
    }
}

fn boolean(vm: &VirtualMachine, name: &str) -> bool {
    match global(vm, name) {
        Value::Boolean(value) => value,
        other => panic!("{name}: booléen attendu, reçu {other:?}"),
    }
}

#[test]
fn match_supports_literals_or_patterns_and_guards() {
    let (vm, result) = run_script(
        r#"
        func classify(value: int) -> int {
            let result = 0;

            match value {
                1 | 2 => { result = 10; }
                x if x > 10 => { result = 20; }
                _ => { result = 30; }
            }

            return result;
        }

        let a = classify(1);
        let b = classify(15);
        let c = classify(7);
        "#,
    );

    result.expect("match avec OR et garde doit s'exécuter");
    assert_eq!(integer(&vm, "a"), 10);
    assert_eq!(integer(&vm, "b"), 20);
    assert_eq!(integer(&vm, "c"), 30);
}

#[test]
fn match_supports_ranges() {
    let (vm, result) = run_script(
        r#"
        func classify(value: int) -> int {
            let result = 0;
            match value {
                0..10 => { result = 1; }
                10..=20 => { result = 2; }
                _ => { result = 3; }
            }
            return result;
        }

        let a = classify(0);
        let b = classify(9);
        let c = classify(10);
        let d = classify(20);
        let e = classify(21);
        "#,
    );

    result.expect("ranges de match doivent s'exécuter");
    assert_eq!(integer(&vm, "a"), 1);
    assert_eq!(integer(&vm, "b"), 1);
    assert_eq!(integer(&vm, "c"), 2);
    assert_eq!(integer(&vm, "d"), 2);
    assert_eq!(integer(&vm, "e"), 3);
}

#[test]
fn match_supports_option_with_bindings() {
    let (vm, result) = run_script(
        r#"
        func read(value: Option<int>) -> int {
            let result = 0;
            match value {
                Some(x) => { result = x; }
                None => { result = -1; }
            }
            return result;
        }

        let some = read(Some(42));
        let none = read(None);
        "#,
    );

    result.expect("match Option doit s'exécuter");
    assert_eq!(integer(&vm, "some"), 42);
    assert_eq!(integer(&vm, "none"), -1);
}

#[test]
fn match_supports_result_with_bindings() {
    let (vm, result) = run_script(
        r#"
        func read(value: Result<int, str>) -> int {
            let result = 0;
            match value {
                Ok(x) => { result = x; }
                Err(error) => {
                    if error == "bad" {
                        result = -1;
                    } else {
                        result = -2;
                    }
                }
            }
            return result;
        }

        let ok = read(Ok(42));
        let err = read(Err("bad"));
        "#,
    );

    result.expect("match Result doit s'exécuter");
    assert_eq!(integer(&vm, "ok"), 42);
    assert_eq!(integer(&vm, "err"), -1);
}

#[test]
fn match_supports_enum_variants() {
    let (vm, result) = run_script(
        r#"
        enum Color {
            Red,
            Green,
            Blue
        }

        func score(color: Color) -> int {
            let result = 0;
            match color {
                Color.Red => { result = 1; }
                Color.Green => { result = 2; }
                Color.Blue => { result = 3; }
            }
            return result;
        }

        let red = score(Color.Red);
        let blue = score(Color.Blue);
        "#,
    );

    result.expect("match enum doit s'exécuter");
    assert_eq!(integer(&vm, "red"), 1);
    assert_eq!(integer(&vm, "blue"), 3);
}

#[test]
fn match_supports_tuple_and_list_rest_patterns() {
    let (vm, result) = run_script(
        r#"
        let tuple_result = 0;
        let tuple = (10, 20);

        match tuple {
            (a, b) => { tuple_result = a + b; }
        }

        let list_result = 0;
        let values = [10, 20, 30];

        match values {
            [head, second, ..] => { list_result = head + second; }
            _ => { list_result = -1; }
        }
        "#,
    );

    result.expect("patterns tuple/liste doivent s'exécuter");
    assert_eq!(integer(&vm, "tuple_result"), 30);
    assert_eq!(integer(&vm, "list_result"), 30);
}

#[test]
fn match_or_bindings_must_have_the_same_names() {
    assert!(
        compile_only(
            r#"
        func value(input: Option<int>) -> int {
            match input {
                Some(x) | Some(y) => { return x; }
                None => { return 0; }
            }
        }
        "#,
        )
        .is_err()
    );

    assert!(
        compile_only(
            r#"
        func value(input: Option<int>) -> int {
            match input {
                Some(x) | Some(x) => { return x; }
                None => { return 0; }
            }
        }
        "#,
        )
        .is_ok()
    );
}

#[test]
fn match_requires_exhaustiveness_for_closed_types() {
    assert!(
        compile_only(
            r#"
        func value(input: Option<int>) -> int {
            match input {
                Some(x) => { return x; }
            }
            return 0;
        }
        "#,
        )
        .is_err()
    );

    assert!(
        compile_only(
            r#"
        func value(input: bool) -> int {
            match input {
                true => { return 1; }
            }
            return 0;
        }
        "#,
        )
        .is_err()
    );
}

#[test]
fn guarded_arms_do_not_make_a_match_exhaustive() {
    assert!(
        compile_only(
            r#"
        func value(input: Option<int>) -> int {
            match input {
                Some(x) if x > 10 => { return x; }
                None => { return 0; }
            }
            return -1;
        }
        "#,
        )
        .is_err()
    );
}

#[test]
fn wildcard_makes_closed_match_exhaustive() {
    let (vm, result) = run_script(
        r#"
        func value(input: Option<int>) -> bool {
            match input {
                Some(_) => { return true; }
                _ => { return false; }
            }
        }

        let some = value(Some(1));
        let none = value(None);
        "#,
    );

    result.expect("le wildcard doit rendre le match exhaustif");
    assert!(boolean(&vm, "some"));
    assert!(!boolean(&vm, "none"));
}
