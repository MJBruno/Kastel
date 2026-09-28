use std::rc::Rc;

use crate::{
    compiler::compiler::Compiler,
    error::compile_error::CompileError,
    frontend::{lexer::lexer::Lexer, parser::Parser},
    runtime::value::Value,
    stdlib::execute_native,
    vm::machine::VirtualMachine,
};

fn run_script(
    source: &str,
) -> (
    VirtualMachine,
    Result<(), crate::error::runtime_error::RuntimeError>,
) {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .unwrap();

    let statements = Parser::new(tokens)
        .parse()
        .unwrap();

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(
        compiler
            .compile(&statements)
            .unwrap(),
    );

    let mut vm = VirtualMachine::new(function, None);
    let result = vm.run();

    (vm, result)
}

fn compile_only(source: &str) -> Result<(), CompileError> {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .unwrap();

    let statements = Parser::new(tokens)
        .parse()
        .unwrap();

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    compiler.compile(&statements).map(|_| ())
}

fn is_wrong_argument_type(error: &CompileError) -> bool {
    match error {
        CompileError::WrongArgumentType { .. } => true,

        CompileError::WithLocation { source, .. } => {
            is_wrong_argument_type(source)
        }

        _ => false,
    }
}

#[test]
fn spawn_and_join_return_value() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            return 42;
        }

        let task = spawn(worker);
        let result = task.join();
        let ok = result == 42;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn sleep_is_a_valid_cooperative_point() {
    let (vm, result) = run_script(
        r#"
        let log = [];

        func worker() -> int {
            log.add(1);
            sleep(5);
            log.add(2);
            return 7;
        }

        let task = spawn(worker);
        let result = task.join();
        let ok = result == 7 && log.size() == 2;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn sleep_zero_yields_to_another_ready_task() {
    let (vm, result) = run_script(
        r#"
        let log = [];

        func first() {
            log.add(1);
            sleep(0);
            log.add(3);
        }

        func second() {
            log.add(2);
        }

        let a = spawn(first);
        let b = spawn(second);
        a.join();
        b.join();
        let ok = log[0] == 1 && log[1] == 2 && log[2] == 3;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn sleep_zero_does_not_deadlock_and_resumes() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            sleep(0);
            return 11;
        }

        let task = spawn(worker);
        let result = task.join();
        let ok = result == 11 && task.status() == "done";
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn root_sleep_allows_ready_tasks_to_progress() {
    let (vm, result) = run_script(
        r#"
        let done = false;

        func worker() {
            done = true;
        }

        spawn(worker);
        sleep(5);
        let ok = done;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn cancelling_a_sleeping_task_removes_its_timer() {
    let (vm, result) = run_script(
        r#"
        let holder = [];
        let log = [];

        func victim() {
            sleep(20);
            log.add(1);
        }

        func killer() {
            holder[0].cancel();
        }

        let victim_task = spawn(victim);
        holder.add(victim_task);
        let killer_task = spawn(killer);
        killer_task.join();

        let caught = false;
        try {
            victim_task.join();
        } catch (e: Err) {
            caught = e.kind == "TaskCancelled";
        }

        let ok = caught && victim_task.status() == "cancelled" && log.size() == 0;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn sleep_requires_an_integer_duration() {
    let error = compile_only(
        r#"
        sleep("10");
    "#,
    )
    .expect_err("sleep should reject non-integer durations");

    assert!(is_wrong_argument_type(&error));
}

#[test]
fn yield_is_a_valid_cooperative_point() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            yield();
            return 7;
        }

        let task = spawn(worker);
        let result = task.join();
        let ok = result == 7;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn task_status_is_exposed() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            return 3;
        }

        let task = spawn(worker);
        let value = task.join();
        let ok = task.is_done() && task.status() == "done" && value == 3;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn spawn_passes_arguments_to_task() {
    let (vm, result) = run_script(
        r#"
        func worker(value: int) -> int {
            return value * 2;
        }

        let task = spawn(worker, 21);
        let result = task.join();
        let ok = result == 42;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn scheduler_round_robin_runs_other_ready_tasks() {
    let (vm, result) = run_script(
        r#"
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
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn task_capture_is_rejected() {
    let (_vm, result) = run_script(
        r#"
        func make_task() {
            let value = 10;
            let task = spawn(() => value + 1);
            task;
        }

        make_task();
    "#,
    );

    let error = result.expect_err(
        "captured task must be rejected",
    );

    assert!(matches!(
        error,
        crate::error::runtime_error::RuntimeError::TaskCaptureNotAllowed
    ));
}

#[test]
fn task_errors_can_be_caught() {
    let (vm, result) = run_script(
        r#"
        func make_task() -> int {
            return spawn(() => 10 << -1).join();
        }

        let caught = false;

        try {
            make_task();
        } catch (e: Err) {
            caught = e.kind == "InvalidShiftAmount";
        }
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("caught"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn scheduler_quantum_preempts_without_explicit_yield() {
    let (vm, result) = run_script(
        r#"
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
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn yield_outside_task_is_rejected() {
    let (_vm, result) = run_script(
        r#"
        yield();
    "#,
    );

    let error = result.expect_err(
        "yield outside a task must fail",
    );

    assert!(matches!(
        error,
        crate::error::runtime_error::RuntimeError::YieldOutsideTask
    ));
}

#[test]
fn channel_send_and_try_recv() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();
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
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_is_shared_between_tasks() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();

        func producer() -> int {
            ch.send(42);
            return 1;
        }

        let task = spawn(producer);
        let task_result = task.join();
        let received = ch.try_recv();

        let ok = task_result == 1 && received == Some(42);
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_values_are_gc_roots() {
    let (vm, result) = run_script(
        r#"
        let ch = channel();

        {
            let value = [1, 2, 3];
            ch.send(value);
        }

        let received = ch.try_recv();
        let ok = received.is_some();
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_recv_blocks_and_send_wakes_waiting_task() {
    let (vm, result) = run_script(
        r#"
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
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_recv_waits_instead_of_busy_polling() {
    let (vm, result) = run_script(
        r#"
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
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_recv_from_root_pumps_scheduler() {
    let (vm, result) = run_script(
        r#"
        let ch: Channel<int> = channel();

        func producer() -> int {
            ch.send(42);
            return 7;
        }

        let task = spawn(producer);

        let received = ch.recv();
        let producer_result = task.join();

        let ok = received == 42 && producer_result == 7;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_wakes_waiters_in_fifo_order() {
    let (vm, result) = run_script(
        r#"
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
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_generic_annotation_checks_send_type_and_recv_result() {
    let (vm, result) = run_script(
        r#"
        let ch: Channel<int> = channel();

        ch.send(42);

        let value: int = ch.recv();
        let maybe: Option<int> = ch.try_recv();

        let ok = value == 42 && maybe.is_none();
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_generic_constructor_accepts_explicit_type_argument() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();

        ch.send(7);

        let value = ch.recv();
        let ok = value == 7;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_generic_annotation_rejects_wrong_send_type() {
    let error = compile_only(
        r#"
        let ch: Channel<int> = channel();
        ch.send("wrong type");
    "#,
    )
    .expect_err("Channel<int> must reject send(str)");

    assert!(
        is_wrong_argument_type(&error),
        "expected WrongArgumentType, got: {error:?}"
    );
}
// ============================================================
//   RÉGRESSION : la VM qui attend (`join` / `recv`) doit rester racine
// ============================================================
//
// Pendant `task.join()`, la tâche tourne (et peut déclencher le GC) alors que
// la pile de la VM appelante n'est PAS dans le registre du scheduler. Sans
// épinglage, un tableau détenu uniquement par une variable locale de
// l'appelant était vidé (`break_cycle`) : `data.size()` renvoyait 0.
#[test]
fn locals_of_the_joining_vm_survive_gc_run_by_the_task() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            let junk = [];
            for i in range(0, 20000) { junk.add([i]); }
            return junk.size();
        }

        func main() -> int {
            let data = [1, 2, 3];
            let task = spawn(worker);
            let produced = task.join();
            return data.size() + produced;
        }

        let total = main();
        let ok = total == 20003;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

// ============================================================
//   TÂCHES JAMAIS JOINTES : elles s'exécutent, puis sont libérées
// ============================================================

fn array_len(vm: &VirtualMachine, name: &str) -> usize {
    let globals = vm.globals.borrow();

    match globals.get(name) {
        Some(Value::Object(handle)) => match &*handle.borrow() {
            crate::runtime::object::Object::Array(items) => items.len(),
            _ => panic!("`{name}` n'est pas un tableau"),
        },
        _ => panic!("`{name}` est absent ou n'est pas un objet"),
    }
}

#[test]
fn an_unjoined_task_still_runs_before_the_program_ends() {
    let (vm, result) = run_script(
        r#"
        let log = [];

        func worker() {
            log.add(1);
        }

        spawn(worker);
        spawn(worker);
    "#,
    );

    result.unwrap();

    assert_eq!(array_len(&vm, "log"), 2);
}

#[test]
fn finished_tasks_release_their_stack_and_detached_ones_their_slot() {
    let (vm, result) = run_script(
        r#"
        let log = [];

        func worker() {
            let scratch = [1, 2, 3];
            log.add(scratch.size());
        }

        let kept = spawn(worker);
        kept.join();

        for i in range(0, 200) { spawn(worker); }
    "#,
    );

    result.unwrap();

    assert_eq!(array_len(&vm, "log"), 201);

    let scheduler = vm.scheduler_owner.as_ref().unwrap().borrow();

    // Tâche encore référencée par `kept` : résultat conservé, pile libérée.
    let kept = scheduler.tasks[0].as_ref().expect("la tâche gardée existe");
    assert!(kept.vm.stack.is_empty());

    // Les tâches sans handle ont libéré leur slot (la dernière peut encore être
    // référencée par le dernier résultat d'expression de la VM).
    let still_held = scheduler.tasks[1..].iter().filter(|slot| slot.is_some()).count();
    assert!(still_held <= 1, "{still_held} slots non libérés");
}


#[test]
fn task_can_be_cancelled_before_it_runs() {
    let (vm, result) = run_script(
        r#"
        let ran = false;

        func worker() {
            ran = true;
            return 42;
        }

        let task = spawn(worker);
        task.cancel();

        let ok = task.is_done()
            && task.status() == "cancelled"
            && ran == false;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn waiting_task_can_be_cancelled() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();

        func consumer() -> int {
            return ch.recv();
        }

        func starter() -> int {
            return 1;
        }

        let consumer_task = spawn(consumer);
        let starter_task = spawn(starter);

        starter_task.join();
        let waiting = consumer_task.status();

        consumer_task.cancel();

        let ok = waiting == "waiting"
            && consumer_task.status() == "cancelled"
            && consumer_task.is_done();
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn cancellation_requested_by_another_task_stops_the_target() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            let i = 0;

            while i < 100000 {
                i = i + 1;
                yield();
            }

            return 42;
        }

        let target: Task<int> = spawn(worker);

        func controller() -> int {
            target.cancel();
            return 1;
        }

        let controller_task = spawn(controller);

        let controller_result = controller_task.join();
        let caught = false;

        try {
            target.join();
        } catch (e: Err) {
            caught = e.kind == "TaskCancelled";
        }

        let ok = controller_result == 1
            && target.status() == "cancelled"
            && caught;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}


#[test]
fn channel_close_is_idempotent_and_send_is_rejected() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();

        let open_before = !ch.is_closed();
        ch.close();
        ch.close();

        let closed_after = ch.is_closed();
        let send_failed = false;

        try {
            ch.send(1);
        } catch (e: Err) {
            send_failed = e.kind == "ChannelClosed";
        }

        let empty = ch.try_recv() == None;
        let ok = open_before && closed_after && send_failed && empty;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn channel_close_preserves_buffered_values() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();
        ch.send(10);
        ch.send(20);
        ch.close();

        let first = ch.recv();
        let second = ch.recv();
        let failed = false;

        try {
            ch.recv();
        } catch (e: Err) {
            failed = e.kind == "ChannelClosed";
        }

        let ok = first == 10 && second == 20 && failed;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn closing_channel_wakes_waiting_consumer_with_catchable_error() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();
        let observed = "waiting";

        func consumer() -> str {
            try {
                ch.recv();
                return "unexpected";
            } catch (e: Err) {
                observed = e.kind;
                return "closed";
            }
        }

        func closer() {
            yield();
            ch.close();
        }

        let task = spawn(consumer);
        let closer_task = spawn(closer);

        let result = task.join();
        let closer_result = closer_task.join();
        let ok = result == "closed"
            && closer_result == None
            && observed == "ChannelClosed"
            && task.status() == "done"
            && ch.is_closed();
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}


#[test]
fn select_returns_first_ready_channel_in_deterministic_order() {
    let (vm, result) = run_script(
        r#"
        let first = channel<int>();
        let second = channel<int>();
        second.send(42);

        let selected = select([first, second]);
        let ok = selected[0] == 1
            && selected[1] == 42
            && selected[2] == false;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn select_blocks_until_one_channel_receives() {
    let (vm, result) = run_script(
        r#"
        let first = channel<int>();
        let second = channel<int>();

        func consumer() -> int {
            let selected = select([first, second]);
            if selected[0] == 1 {
                return selected[1];
            }
            return -1;
        }

        func producer() -> int {
            yield();
            second.send(77);
            first.send(99);
            return 1;
        }

        let consumer_task = spawn(consumer);
        let producer_task = spawn(producer);

        let consumer_result = consumer_task.join();
        let producer_result = producer_task.join();
        let buffered = first.try_recv();
        let ok = consumer_result == 77
            && producer_result == 1
            && buffered == Some(99);
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn closing_a_waited_select_wakes_it_as_a_closed_case() {
    let (vm, result) = run_script(
        r#"
        let first = channel<int>();
        let second = channel<int>();

        func consumer() -> int {
            let selected = select([first, second]);
            if selected[0] == 1 && selected[1] == None && selected[2] == true {
                return 7;
            }
            return -1;
        }

        func closer() -> int {
            yield();
            second.close();
            return 1;
        }

        let consumer_task = spawn(consumer);
        let closer_task = spawn(closer);
        let consumer_result = consumer_task.join();
        let closer_result = closer_task.join();
        let ok = consumer_result == 7 && closer_result == 1;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn select_reports_closed_channel_without_losing_which_channel_closed() {
    let (vm, result) = run_script(
        r#"
        let first = channel<int>();
        let second = channel<int>();
        second.close();

        let selected = select([first, second]);
        let ok = selected[0] == 1
            && selected[1] == None
            && selected[2] == true;
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn select_with_no_progress_is_reported_as_deadlock() {
    let (_vm, result) = run_script(
        r#"
        let first = channel<int>();
        let second = channel<int>();

        func worker() -> int {
            let selected = select([first, second]);
            return selected[0];
        }

        let task = spawn(worker);
        task.join();
    "#,
    );

    assert!(matches!(
        result,
        Err(crate::error::runtime_error::RuntimeError::TaskDeadlock)
    ));
}

#[test]
fn cancelling_select_removes_all_channel_wait_registrations() {
    let (vm, result) = run_script(
        r#"
        let first = channel<int>();
        let second = channel<int>();

        func worker() -> int {
            let selected = select([first, second]);
            return selected[0];
        }

        let task = spawn(worker);

        func starter() -> int {
            yield();
            return 1;
        }

        let starter_task = spawn(starter);
        starter_task.join();
        task.cancel();

        first.send(10);
        second.send(20);

        let ok = task.status() == "cancelled"
            && first.try_recv() == Some(10)
            && second.try_recv() == Some(20);
    "#,
    );

    result.unwrap();

    assert!(matches!(
        vm.globals.borrow().get("ok"),
        Some(Value::Boolean(true))
    ));
}

#[test]
fn multiple_tasks_waiting_without_progress_are_reported_as_deadlock() {
    let (_vm, result) = run_script(
        r#"
        let first_channel = channel<int>();
        let second_channel = channel<int>();

        func first_worker() -> int {
            return first_channel.recv();
        }

        func second_worker() -> int {
            return second_channel.recv();
        }

        let first = spawn(first_worker);
        let _second = spawn(second_worker);

        first.join();
    "#,
    );

    assert!(matches!(
        result,
        Err(crate::error::runtime_error::RuntimeError::TaskDeadlock)
    ));
}

// ============================================================
//   PHASE 3 : ROBUSTESSE DU SCHEDULER
//   (tâches imbriquées, cas limites d'annulation, stress)
// ============================================================

fn assert_global_true(vm: &VirtualMachine, name: &str) {
    assert!(
        matches!(vm.globals.borrow().get(name), Some(Value::Boolean(true))),
        "`{name}` devrait valoir true"
    );
}

/// Exécute `f` sur un thread à grande pile : les `join` imbriqués récursent
/// sur la pile native, et les threads de test n'ont que 2 Mio par défaut.
fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

// ---------- tâches imbriquées ----------

#[test]
fn nested_tasks_can_spawn_and_join_children() {
    let (vm, result) = run_script(
        r#"
        func leaf(value: int) -> int {
            return value + 1;
        }

        func middle(value: int) -> int {
            let child = spawn(leaf, value);
            return child.join() * 2;
        }

        func top() -> int {
            let child = spawn(middle, 4);
            return child.join() + 100;
        }

        let result = spawn(top).join();
        let ok = result == 110;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn children_outlive_a_parent_that_never_joins_them() {
    let (vm, result) = run_script(
        r#"
        let log = [];

        func child() {
            log.add(1);
        }

        func parent() -> int {
            spawn(child);
            spawn(child);
            return 7;
        }

        let value = spawn(parent).join();
        let ok = value == 7;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
    // Les enfants sans handle tournent quand même avant la fin du programme.
    assert_eq!(array_len(&vm, "log"), 2);
}

#[test]
fn nested_join_chain_within_the_limit_completes() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            func chain(n: int) -> int {
                if n == 0 {
                    return 0;
                }

                let child = spawn(chain, n - 1);
                return child.join() + 1;
            }

            let result = spawn(chain, 20).join();
            let ok = result == 20;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn nested_join_chain_beyond_the_limit_is_a_catchable_error() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            func chain(n: int) -> int {
                if n == 0 {
                    return 0;
                }

                let child = spawn(chain, n - 1);
                return child.join() + 1;
            }

            let caught = false;

            try {
                spawn(chain, 200).join();
            } catch (e: Err) {
                caught = e.kind == "TaskNestingTooDeep";
            }

            let ok = caught;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn mutual_join_between_nested_tasks_is_a_deadlock_error() {
    let (vm, result) = run_script(
        r#"
        let holder = [];

        func first() -> int {
            return holder[1].join();
        }

        func second() -> int {
            return holder[0].join();
        }

        let a = spawn(first);
        let b = spawn(second);
        holder.add(a);
        holder.add(b);

        let caught = false;

        try {
            a.join();
        } catch (e: Err) {
            caught = e.kind == "TaskDeadlock";
        }

        let ok = caught;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn a_task_joining_itself_is_a_deadlock_error() {
    let (vm, result) = run_script(
        r#"
        let holder = [];

        func worker() -> int {
            return holder[0].join();
        }

        let task = spawn(worker);
        holder.add(task);

        let caught = false;

        try {
            task.join();
        } catch (e: Err) {
            caught = e.kind == "TaskDeadlock";
        }

        let ok = caught;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

// ---------- cas limites d'annulation ----------

#[test]
fn cancelling_a_finished_task_keeps_its_result() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            return 5;
        }

        let task = spawn(worker);
        let first = task.join();
        task.cancel();

        let ok = first == 5
            && task.status() == "done"
            && task.join() == 5;
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn cancelling_twice_is_idempotent() {
    let (vm, result) = run_script(
        r#"
        func worker() -> int {
            return 1;
        }

        let task = spawn(worker);
        task.cancel();
        task.cancel();

        let ok = task.status() == "cancelled" && task.is_done();
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn a_task_cancelling_itself_stops_at_its_next_yield() {
    let (vm, result) = run_script(
        r#"
        let holder = [];
        let log = [];

        func worker() -> int {
            log.add(1);
            holder[0].cancel();
            yield();
            log.add(2);
            return 1;
        }

        let task = spawn(worker);
        holder.add(task);

        let caught = false;

        try {
            task.join();
        } catch (e: Err) {
            caught = e.kind == "TaskCancelled";
        }

        let ok = caught && task.status() == "cancelled";
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
    assert_eq!(array_len(&vm, "log"), 1);
}

#[test]
fn a_late_cancellation_does_not_discard_a_completed_result() {
    let (vm, result) = run_script(
        r#"
        let holder = [];

        func killer() -> int {
            holder[0].cancel();
            return 1;
        }

        func victim() -> int {
            let helper = spawn(killer);
            let value = helper.join();
            return value + 41;
        }

        let task = spawn(victim);
        holder.add(task);

        let result = task.join();
        let ok = result == 42 && task.status() == "done";
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");
}

#[test]
fn cancellation_requested_before_blocking_leaves_no_ghost_waiter() {
    let (vm, result) = run_script(
        r#"
        let ch = channel<int>();
        let holder = [];

        func controller() -> int {
            holder[0].cancel();
            return 1;
        }

        func worker() -> int {
            let helper = spawn(controller);
            helper.join();
            return ch.recv();
        }

        let task = spawn(worker);
        holder.add(task);

        let caught = false;

        try {
            task.join();
        } catch (e: Err) {
            caught = e.kind == "TaskCancelled";
        }

        ch.send(9);
        let ok = caught
            && task.status() == "cancelled"
            && ch.try_recv() == Some(9);
    "#,
    );

    result.unwrap();
    assert_global_true(&vm, "ok");

    let scheduler = vm.scheduler_owner.as_ref().unwrap().borrow();
    assert!(scheduler.waiting_channels.is_empty());
    assert!(scheduler.waiting_selects.is_empty());
    assert!(scheduler.cancel_requested.is_empty());
}


#[test]
fn select_with_timeout_returns_timeout_tuple() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let ch = channel<int>();
            let selected = select([ch], 5);
            let ok = selected[0] == -1 && selected[1] == None && selected[2] == false;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn select_wakes_on_channel_before_timeout() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let ch = channel<int>();

            func sender() -> int {
                sleep(5);
                ch.send(42);
                return 1;
            }

            spawn(sender);
            let selected = select([ch], 1000);
            let ok = selected[0] == 0 && selected[1] == 42 && selected[2] == false;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn select_timeout_cleans_all_channel_registrations() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let first = channel<int>();
            let second = channel<int>();

            func waiter() -> int {
                let selected = select([first, second], 5);
                return selected[0];
            }

            let task = spawn(waiter);
            let result = task.join();
            let ok = result == -1;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");

        let scheduler = vm.scheduler_owner.as_ref().unwrap().borrow();
        assert!(scheduler.waiting_channels.is_empty());
        assert!(scheduler.waiting_selects.is_empty());
        assert!(scheduler.sleeping_tasks.is_empty());
    });
}

#[test]
fn select_rejects_negative_timeout() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let ch = channel<int>();
            let ok = false;
            try {
                select([ch], -1);
            } catch (e: Err) {
                ok = e.kind == "TypeError";
            }
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

// ---------- tests de stress ----------

#[test]
fn stress_many_tasks_all_complete() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            func worker(n: int) -> int {
                yield();
                return n;
            }

            let tasks = [];

            for i in range(0, 500) {
                tasks.add(spawn(worker, i));
            }

            let sum = 0;

            for t in tasks {
                sum = sum + t.join();
            }

            let ok = sum == 124750;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn stress_producer_consumer_through_a_channel() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let ch = channel<int>();

            func producer(count: int) -> int {
                for i in range(0, count) {
                    ch.send(i);

                    if i % 50 == 0 {
                        yield();
                    }
                }

                ch.close();
                return count;
            }

            func consumer() -> int {
                let sum = 0;
                let running = true;

                while running {
                    try {
                        sum = sum + ch.recv();
                    } catch (e: Err) {
                        running = false;
                    }
                }

                return sum;
            }

            let p = spawn(producer, 2000);
            let c = spawn(consumer);

            let produced = p.join();
            let consumed = c.join();

            let ok = produced == 2000 && consumed == 1999000;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn stress_cancelling_many_waiting_tasks_cleans_every_registration() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let ch = channel<int>();

            func waiter() -> int {
                return ch.recv();
            }

            func pump() -> int {
                yield();
                return 1;
            }

            let tasks = [];

            for i in range(0, 200) {
                tasks.add(spawn(waiter));
            }

            spawn(pump).join();

            let cancelled = 0;

            for t in tasks {
                t.cancel();

                if t.status() == "cancelled" {
                    cancelled = cancelled + 1;
                }
            }

            ch.send(1);

            let ok = cancelled == 200 && ch.size() == 1;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");

        let scheduler = vm.scheduler_owner.as_ref().unwrap().borrow();
        assert!(scheduler.waiting_channels.is_empty());
        assert!(scheduler.waiting_selects.is_empty());
    });
}

#[test]
fn stress_gc_pressure_across_many_tasks_keeps_root_data_alive() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            func churn(n: int) -> int {
                let junk = [];

                for i in range(0, 2000) {
                    junk.add([i, n]);
                }

                return junk.size();
            }

            let keep = [[1], [2], [3]];
            let tasks = [];

            for i in range(0, 30) {
                tasks.add(spawn(churn, i));
            }

            let total = 0;

            for t in tasks {
                total = total + t.join();
            }

            let ok = total == 60000 && keep.size() == 3 && keep[0][0] == 1;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn mutex_try_lock_and_unlock() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let m = mutex();

            func worker() -> bool {
                let first = m.try_lock();
                let second = m.try_lock();
                m.unlock();
                return first && !second;
            }

            let task = spawn(worker);
            let ok = task.join();
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn mutex_waiters_are_fifo_and_resume_after_unlock() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let m = mutex();
            let events = channel<int>();

            func first() {
                m.lock();
                events.send(1);
                yield();
                m.unlock();
            }

            func second() {
                m.lock();
                events.send(2);
                m.unlock();
            }

            let a = spawn(first);
            events.recv();
            let b = spawn(second);

            let first_done = a.join();
            let second_event = events.recv();
            let second_done = b.join();
            let ok = second_event == 2 && m.is_locked() == false;
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn mutex_non_reentrant_lock_is_a_catchable_error() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let m = mutex();

            func worker() -> bool {
                m.lock();

                try {
                    m.lock();
                    return false;
                } catch (e: Err) {
                    m.unlock();
                    return e.kind == "MutexDeadlock";
                }
            }

            let ok = spawn(worker).join();
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn mutex_unlock_requires_the_owner() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let m = mutex();
            let gate = channel<int>();

            func holder() {
                m.lock();
                gate.send(1);
                yield();
                m.unlock();
            }

            func intruder() -> str {
                try {
                    m.unlock();
                    return "bad";
                } catch (e: Err) {
                    return e.kind;
                }
            }

            let owner = spawn(holder);
            gate.recv();
            let result = spawn(intruder).join();
            owner.cancel();
            let ok = result == "MutexNotOwner";
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}

#[test]
fn cancelling_mutex_owner_releases_the_lock() {
    on_big_stack(|| {
        let (vm, result) = run_script(
            r#"
            let m = mutex();
            let gate = channel<int>();

            func holder() {
                m.lock();
                gate.send(1);
                while true {
                    yield();
                }
            }

            func waiter() {
                m.lock();
                gate.send(2);
                m.unlock();
            }

            let owner = spawn(holder);
            gate.recv();
            let waiter_task = spawn(waiter);
            owner.cancel();

            let acquired = gate.recv();
            waiter_task.join();
            let ok = acquired == 2 && !m.is_locked();
        "#,
        );

        result.unwrap();
        assert_global_true(&vm, "ok");
    });
}
