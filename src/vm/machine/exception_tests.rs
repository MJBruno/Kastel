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
