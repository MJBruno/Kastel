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
fn option_core_api() {
    let source = r#"
        func increment(x: int) -> int {
            return x + 1;
        }

        func positive(x: int) -> Option<int> {
            if x > 0 {
                return Some(x * 2);
            }
            return None;
        }

        let some = Some(41);
        let none: Option<int> = None;

        let value = some.unwrap();
        let mapped = some.map(increment).unwrap();
        let chained = some.and_then(positive).unwrap();
        let fallback = none.unwrap_or(7);
        let missing = none.ok_or("missing").unwrap_err();
        let some_state = some.is_some();
        let none_state = none.is_none();
        let text = some.to_string();
        let empty_text = none.to_string();

        let cyclic_list = [];
        let cyclic_option = Some(cyclic_list);
        cyclic_list.add(cyclic_option);
        let cyclic_text = cyclic_option.to_string();
    "#;

    let (vm, result) = run_script(source);
    result.expect("Option doit s'exécuter");

    assert_eq!(integer(&vm, "value"), 41);
    assert_eq!(integer(&vm, "mapped"), 42);
    assert_eq!(integer(&vm, "chained"), 82);
    assert_eq!(integer(&vm, "fallback"), 7);
    assert_eq!(global(&vm, "missing").as_string_value().as_deref(), Some("missing"));
    assert!(boolean(&vm, "some_state"));
    assert!(boolean(&vm, "none_state"));
    assert_eq!(global(&vm, "text").as_string_value().as_deref(), Some("Some(41)"));
    assert_eq!(global(&vm, "empty_text").as_string_value().as_deref(), Some("None"));
    assert!(
        global(&vm, "cyclic_text")
            .as_string_value()
            .is_some_and(|text| text.contains("...")),
        "Option/Result doit participer à la protection anti-cycle de l'affichage"
    );
}

#[test]
fn result_core_api() {
    let source = r#"
        func increment(x: int) -> int {
            return x + 1;
        }

        func double_ok(x: int) -> Result<int, str> {
            return Ok(x * 2);
        }

        let ok: Result<int, str> = Ok(41);
        let err: Result<int, str> = Err("bad");

        let value = ok.unwrap();
        let mapped = ok.map(increment).unwrap();
        let chained = ok.and_then(double_ok).unwrap();
        let fallback = err.unwrap_or(7);
        let mapped_error = err.map_err(func(message) { return message + "!"; }).unwrap_err();
        let ok_value = ok.ok().unwrap();
        let err_value = err.err().unwrap();
        let ok_state = ok.is_ok();
        let err_state = err.is_err();
        let text = ok.to_string();
        let err_text = err.to_string();
    "#;

    let (vm, result) = run_script(source);
    result.expect("Result doit s'exécuter");

    assert_eq!(integer(&vm, "value"), 41);
    assert_eq!(integer(&vm, "mapped"), 42);
    assert_eq!(integer(&vm, "chained"), 82);
    assert_eq!(integer(&vm, "fallback"), 7);
    assert_eq!(
        global(&vm, "mapped_error").as_string_value().as_deref(),
        Some("bad!")
    );
    assert_eq!(integer(&vm, "ok_value"), 41);
    assert_eq!(
        global(&vm, "err_value").as_string_value().as_deref(),
        Some("bad")
    );
    assert!(boolean(&vm, "ok_state"));
    assert!(boolean(&vm, "err_state"));
    assert_eq!(global(&vm, "text").as_string_value().as_deref(), Some("Ok(41)"));
    assert_eq!(
        global(&vm, "err_text").as_string_value().as_deref(),
        Some("Err(bad)")
    );
}

#[test]
fn option_and_result_fail_explicitly_when_unwrapped_wrongly() {
    let (_, option_error) = run_script("let value: Option<int> = None; value.unwrap();");
    assert!(matches!(option_error, Err(RuntimeError::OptionUnwrap { .. })));

    let (_, result_error) = run_script(
        r#"
        let value: Result<int, str> = Err("no value");
        value.expect("expected a success");
        "#,
    );
    assert!(matches!(result_error, Err(RuntimeError::ResultUnwrap { expected: "Ok", .. })));
}

#[test]
fn option_and_result_types_are_checked_statically() {
    assert!(compile_only(
        r#"
        let a: Option<int> = None;
        let b: Option<int> = Some(1);
        let c: Result<int, str> = Ok(1);
        let d: Result<int, str> = Err("error");
        "#,
    )
    .is_ok());

    assert!(compile_only("let value: int = None;").is_err());

    assert!(compile_only(
        r#"
        let value: Option<int> = Some("wrong");
        "#,
    )
    .is_err());

    assert!(compile_only(
        r#"
        let value: Result<int, str> = Err(42);
        "#,
    )
    .is_err());
}
