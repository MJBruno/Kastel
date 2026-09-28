use std::rc::Rc;

use crate::{
    compiler::compiler::Compiler,
    error::compile_error::CompileError,
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

fn compile_only(source: &str) -> Result<(), CompileError> {
    let tokens = Lexer::new(source.to_string()).scan_token().unwrap();
    let statements = Parser::new(tokens).parse().unwrap();

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);
    compiler.compile(&statements).map(|_| ())
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

#[test]
fn channel_send_and_try_recv() {
    let (vm, result) = run_script(r#"
        let ch = channel();
        let empty_before = ch.is_empty();
        ch.send(10);
        ch.send(20);
        let size = ch.size();
        let first = ch.try_recv();
        let second = ch.try_recv();
        let empty_after = ch.is_empty();
        let missing = ch.try_recv();

        let ok = empty_before
            && size == 2
            && first == Some(10)
            && second == Some(20)
            && empty_after
            && missing == None;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn channel_is_shared_between_tasks() {
    let (vm, result) = run_script(r#"
        let ch = channel();

        func producer() -> int {
            ch.send(42);
            return 1;
        }

        let task = spawn(producer);
        let task_result = task.join();
        let received = ch.try_recv();

        let ok = task_result == 1 && received == Some(42);
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn channel_values_are_gc_roots() {
    let (vm, result) = run_script(r#"
        let ch = channel();
        {
            let value = [1, 2, 3];
            ch.send(value);
        }
        let received = ch.try_recv();
        let ok = received.is_some();
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}


#[test]
fn channel_recv_blocks_and_send_wakes_waiting_task() {
    let (vm, result) = run_script(r#"
        let ch = channel();
        let log = [];

        func consumer() -> int {
            log.add(1);
            let value = ch.recv();
            log.add(value);
            return value;
        }

        func producer() -> int {
            log.add(2);
            ch.send(42);
            return 7;
        }

        let consumer_task = spawn(consumer);
        let producer_task = spawn(producer);

        let consumer_result = consumer_task.join();
        let producer_result = producer_task.join();

        let ok = consumer_result == 42
            && producer_result == 7
            && log[0] == 1
            && log[1] == 2
            && log[2] == 42
            && consumer_task.status() == "done";
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn channel_recv_waits_instead_of_busy_polling() {
    let (vm, result) = run_script(r#"
        let ch = channel();
        let observed = "";

        func consumer() -> int {
            observed = "waiting";
            return ch.recv();
        }

        let consumer_task = spawn(consumer);
        consumer_task.status();

        func producer() -> int {
            ch.send(99);
            return 1;
        }

        let producer_task = spawn(producer);
        let producer_result = producer_task.join();
        let consumer_result = consumer_task.join();

        let ok = observed == "waiting"
            && producer_result == 1
            && consumer_result == 99;
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}

#[test]
fn channel_recv_outside_task_is_rejected() {
    let (_vm, result) = run_script(r#"
        let ch = channel();
        ch.recv();
    "#);

    let error = result.expect_err("root recv must fail");
    assert!(matches!(
        error,
        crate::error::runtime_error::RuntimeError::ChannelRecvOutsideTask
    ));
}


#[test]
fn channel_wakes_waiters_in_fifo_order() {
    let (vm, result) = run_script(r#"
        let ch = channel();
        let log = [];

        func consumer(id: int) -> int {
            let value = ch.recv();
            log.add(id * 100 + value);
            return value;
        }

        func producer() -> int {
            ch.send(1);
            ch.send(2);
            return 0;
        }

        let first = spawn(consumer, 1);
        let second = spawn(consumer, 2);
        let producer_task = spawn(producer);

        let first_result = first.join();
        let second_result = second.join();
        let producer_result = producer_task.join();

        let ok = first_result == 1
            && second_result == 2
            && producer_result == 0
            && log[0] == 101
            && log[1] == 202
            && ch.is_empty();
    "#);

    result.unwrap();
    assert!(matches!(vm.globals.borrow().get("ok"), Some(Value::Boolean(true))));
}


#[test]
fn channel_generic_annotation_checks_send_type_and_recv_result() {
    let (vm, result) = run_script(r#"
        let ch: Channel<int> = channel();
        ch.send(42);

        let value: int = ch.recv();
        let maybe: Option<int> = ch.try_recv();

        let ok = value == 42 && maybe.is_none();
    "#);

    result.unwrap();
    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_generic_constructor_accepts_explicit_type_argument() {
    let (vm, result) = run_script(r#"
        let ch = channel<int>();
        ch.send(7);
        let value = ch.recv();
        let ok = value == 7;
    "#);

    result.unwrap();
    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}


#[test]
fn channel_generic_annotation_rejects_wrong_send_type() {
    let error = compile_only(r#"
        let ch: Channel<int> = channel();
        ch.send("wrong type");
    "#)
    .expect_err("Channel<int> must reject send(str)");

    assert!(matches!(error, CompileError::WrongArgumentType { .. }));
}
