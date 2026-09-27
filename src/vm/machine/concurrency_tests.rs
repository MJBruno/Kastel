use std::rc::Rc;

use crate::{
    compiler::compiler::Compiler,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

fn run_script(source: &str) -> (VirtualMachine, Result<(), crate::error::runtime_error::RuntimeError>) {
    let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
    let statements = Parser::new(tokens).parse().unwrap();

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(compiler.compile(&statements).unwrap());
    let mut vm = VirtualMachine::new(function, None);
    let result = vm.run();

    (vm, result)
}

#[test]
fn spawn_and_join_return_value() {
    let (vm, result) = run_script(r#"
        func worker() -> int {
            return 42;
        }

        let task = spawn(worker);
        let result = task.join();
        let ok = result == 42;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn yield_is_a_valid_cooperative_point() {
    let (vm, result) = run_script(r#"
        func worker() -> int {
            yield();
            return 7;
        }

        let task = spawn(worker);
        let result = task.join();
        let ok = result == 7;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn task_status_is_exposed() {
    let (vm, result) = run_script(r#"
        func worker() -> int {
            return 3;
        }

        let task = spawn(worker);
        let value = task.join();
        let ok = task.is_done() && task.status() == "done" && value == 3;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}


#[test]
fn spawn_passes_arguments_to_task() {
    let (vm, result) = run_script(r#"
        func worker(value: int) -> int {
            return value * 2;
        }

        let task = spawn(worker, 21);
        let result = task.join();
        let ok = result == 42;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn scheduler_round_robin_runs_other_ready_tasks() {
    let (vm, result) = run_script(r#"
        let log = [];

        func worker(id: int) -> int {
            log.add(id);
            yield();
            log.add(id + 10);
            return id;
        }

        let first = spawn(worker, 1);
        let second = spawn(worker, 2);

        let first_result = first.join();
        let second_result = second.join();

        let ok = first_result == 1
            && second_result == 2
            && log[0] == 1
            && log[1] == 2
            && log[2] == 11
            && log[3] == 12;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn task_capture_is_rejected() {
    let (_vm, result) = run_script(r#"
        func make_task() {
            let value = 10;
            let task = spawn(() => value + 1);
            task;
        }

        make_task();
    "#);

    let error = result.expect_err("captured task must be rejected");
    assert!(matches!(error, crate::error::runtime_error::RuntimeError::TaskCaptureNotAllowed));
}

#[test]
fn task_errors_can_be_caught() {
    let (vm, result) = run_script(r#"
        func make_task() -> int {
            return spawn(() => 10 << -1).join();
        }

        let caught = false;

        try {
            make_task();
        } catch (e: Err) {
            caught = e.kind == "InvalidShiftAmount";
        }
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("caught"), Some(Value::Boolean(true))));
}

#[test]
fn scheduler_quantum_preempts_without_explicit_yield() {
    let (vm, result) = run_script(r#"
        let log = [];

        func busy() -> int {
            log.add(1);

            let i = 0;
            while i < 5000 {
                i = i + 1;
            }

            log.add(3);
            return 3;
        }

        func quick() -> int {
            log.add(2);
            return 2;
        }

        let first = spawn(busy);
        let second = spawn(quick);

        let first_result = first.join();
        let second_result = second.join();

        let ok = first_result == 3
            && second_result == 2
            && log[0] == 1
            && log[1] == 2
            && log[2] == 3;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}


#[test]
fn yield_outside_task_is_rejected() {
    let (_vm, result) = run_script(r#"
        yield();
    "#);

    let error = result.expect_err("yield outside a task must fail");
    assert!(matches!(error, crate::error::runtime_error::RuntimeError::YieldOutsideTask));
}
