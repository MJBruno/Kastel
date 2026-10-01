//! Tests d'intégration de la concurrence de Kastel.
//!
//! Ces tests couvrent le contrat observable du scheduler et des primitives
//! coopératives. Ils utilisent le même pipeline que le runtime réel :
//! lexer -> parser -> compiler -> VM.
//!
//! Principes :
//! - pas de thread OS dans les tests ;
//! - orchestration déterministe avec `yield()` ;
//! - chaque primitive expose au moins son chemin nominal et un chemin d'erreur ;
//! - les assertions portent sur des effets observables dans Kastel ;
//! - les annulations vérifient aussi la libération des ressources.

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
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .expect("le lexer doit accepter le script de concurrence");
    let statements = Parser::new(tokens)
        .parse()
        .expect("le parser doit accepter le script de concurrence");

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);

    let function = Rc::new(
        compiler
            .compile(&statements)
            .expect("le compilateur doit compiler le script de concurrence"),
    );
    let mut vm = VirtualMachine::new(function, None);
    let result = vm.run().map(|_| ());

    (vm, result)
}

fn compile_only(source: &str) {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .expect("le lexer doit accepter le script");
    let statements = Parser::new(tokens)
        .parse()
        .expect("le parser doit accepter le script");

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);
    compiler
        .compile(&statements)
        .expect("le script doit être compilable");
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

fn integer(vm: &VirtualMachine, name: &str) -> i64 {
    match global(vm, name) {
        Value::Integer(value) => value,
        other => panic!("{name}: entier attendu, reçu {other:?}"),
    }
}

fn string(vm: &VirtualMachine, name: &str) -> String {
    global(vm, name)
        .as_string_value()
        .unwrap_or_else(|| panic!("{name}: chaîne attendue"))
}

fn array_len(vm: &VirtualMachine, name: &str) -> usize {
    global(vm, name)
        .array_len()
        .expect("la globale doit être un tableau")
}

fn integer_at(vm: &VirtualMachine, name: &str, index: usize) -> i64 {
    match global(vm, name)
        .array_get(index)
        .expect("index de tableau valide attendu")
    {
        Value::Integer(value) => value,
        other => panic!("{name}[{index}]: entier attendu, reçu {other:?}"),
    }
}

fn string_at(vm: &VirtualMachine, name: &str, index: usize) -> String {
    global(vm, name)
        .array_get(index)
        .expect("index de tableau valide attendu")
        .as_string_value()
        .unwrap_or_else(|| panic!("{name}[{index}]: chaîne attendue"))
}

fn boolean_at(vm: &VirtualMachine, name: &str, index: usize) -> bool {
    match global(vm, name)
        .array_get(index)
        .expect("index de tableau valide attendu")
    {
        Value::Boolean(value) => value,
        other => panic!("{name}[{index}]: booléen attendu, reçu {other:?}"),
    }
}

fn expect_runtime_error(source: &str, expected_kind: &str) {
    let (_, result) = run_script(source);
    let error = result.expect_err("une erreur runtime était attendue");
    assert_eq!(error.kind_name(), expected_kind);
}

fn expect_compile_error(source: &str) {
    let tokens = Lexer::new(source.to_string())
        .scan_token()
        .expect("le lexer doit accepter le script");
    let statements = Parser::new(tokens)
        .parse()
        .expect("le parser doit accepter le script");

    let mut compiler = Compiler::new();
    execute_native(&mut compiler);
    assert!(
        compiler.compile(&statements).is_err(),
        "une erreur de compilation était attendue"
    );
}

// ============================================================================
// TASKS / SCHEDULER
// ============================================================================

#[test]
fn spawn_then_join_returns_task_result() {
    let (vm, result) = run_script(
        r#"
        func work() -> int { return 42; }
        let t = spawn(work);
        let value = t.join();
        let status = t.status();
        let done = t.is_done();
        "#,
    );

    result.expect("spawn puis join doit réussir");
    assert_eq!(integer(&vm, "value"), 42);
    assert_eq!(string(&vm, "status"), "done");
    assert!(boolean(&vm, "done"));
}

#[test]
fn task_status_transitions_from_ready_to_waiting_to_done() {
    let (vm, result) = run_script(
        r#"
        let ready = "";
        let waiting = "";
        let value = 0;
        let done = "";

        func worker(ch) -> int {
            let value = ch.recv();
            return value;
        }

        func scenario() {
            let c = channel();
            let t = spawn(worker, c);
            ready = t.status();
            yield();
            waiting = t.status();
            c.send(7);
            value = t.join();
            done = t.status();
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("la transition d'état de Task doit réussir");
    assert_eq!(string(&vm, "ready"), "ready");
    assert_eq!(string(&vm, "waiting"), "waiting");
    assert_eq!(integer(&vm, "value"), 7);
    assert_eq!(string(&vm, "done"), "done");
}



#[test]
fn yield_is_cooperative_and_preserves_fifo_ready_order() {
    let (vm, result) = run_script(
        r#"
        let order = [];

        func worker(id) -> None {
            order.add(id);
            yield();
            order.add(id + 10);
        }

        let a = spawn(worker, 1);
        let b = spawn(worker, 2);
        a.join();
        b.join();
        "#,
    );

    result.expect("yield doit laisser le scheduler reprendre la main");
    assert_eq!(array_len(&vm, "order"), 4);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 11);
    assert_eq!(integer_at(&vm, "order", 3), 12);
}



#[test]
fn sleep_moves_a_task_to_waiting_until_the_timer_expires() {
    let (vm, result) = run_script(
        r#"
        let waiting = "";
        let value = 0;

        func sleeper() -> int {
            sleep(10);
            return 99;
        }

        func scenario() {
            let t = spawn(sleeper);
            yield();
            waiting = t.status();
            value = t.join();
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("sleep doit être coopératif");
    assert_eq!(string(&vm, "waiting"), "waiting");
    assert_eq!(integer(&vm, "value"), 99);
}



#[test]
fn detached_tasks_are_drained_before_vm_shutdown() {
    let (vm, result) = run_script(
        r#"
        let completed = [];

        func background(output) -> None {
            output.add(42);
        }

        spawn(background, completed);
        "#,
    );

    result.expect("une tâche détachée doit être drainée avant l'arrêt de la VM");
    assert_eq!(array_len(&vm, "completed"), 1);
    assert_eq!(integer_at(&vm, "completed", 0), 42);
}

#[test]
fn async_function_call_produces_a_task_and_await_returns_its_result() {
    let (vm, result) = run_script(
        r#"
        async func compute() -> int {
            yield();
            return 7;
        }

        let task = compute();
        let value = await task;
        let status = task.status();
        "#,
    );

    result.expect("une fonction async doit produire un Task<T>");
    assert_eq!(integer(&vm, "value"), 7);
    assert_eq!(string(&vm, "status"), "done");
}

#[test]
fn nested_await_works_inside_a_task() {
    let (vm, result) = run_script(
        r#"
        async func inner() -> int {
            return 5;
        }

        async func outer() -> int {
            let child = inner();
            return await child;
        }

        let task = outer();
        let value = await task;
        "#,
    );

    result.expect("await doit fonctionner dans une tâche enfant");
    assert_eq!(integer(&vm, "value"), 5);
}

#[test]
fn multiple_awaiters_are_woken_when_the_target_task_finishes() {
    let (vm, result) = run_script(
        r#"
        let output = [];
        let a_waiting = "";
        let b_waiting = "";
        let va = 0;
        let vb = 0;

        async func producer() -> int {
            sleep(5);
            return 9;
        }

        func waiter(target, output, id) -> int {
            let value = await target;
            output.add(id + value);
            return value;
        }

        func scenario() {
            let target = producer();
            let a = spawn(waiter, target, output, 1);
            let b = spawn(waiter, target, output, 2);

            yield();
            yield();
            yield();
            a_waiting = a.status();
            b_waiting = b.status();

            target.join();
            va = a.join();
            vb = b.join();
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("plusieurs awaiters doivent être réveillés par la même tâche");
    assert_eq!(string(&vm, "a_waiting"), "waiting");
    assert_eq!(string(&vm, "b_waiting"), "waiting");
    assert_eq!(integer(&vm, "va"), 9);
    assert_eq!(integer(&vm, "vb"), 9);
    assert_eq!(array_len(&vm, "output"), 2);
    assert_eq!(integer_at(&vm, "output", 0) + integer_at(&vm, "output", 1), 21);
}



#[test]
fn task_failure_is_reported_through_join_and_status() {
    let (vm, result) = run_script(
        r#"
        func fail() -> int {
            return 10 % 0;
        }

        let t = spawn(fail);
        let status = "";
        let caught = false;
        let kind = "";

        try {
            t.join();
        } catch (e: Err) {
            caught = true;
            kind = e.kind;
            status = t.status();
        }
        "#,
    );

    result.expect("l'échec de la tâche doit être observable sans casser la VM");
    assert_eq!(string(&vm, "status"), "failed");
    assert!(boolean(&vm, "caught"));
    assert_eq!(string(&vm, "kind"), "DivisionByZero");
}



#[test]
fn cancelling_a_task_twice_is_idempotent_and_marks_it_cancelled() {
    let (vm, result) = run_script(
        r#"
        func work() -> int { return 1; }

        let t = spawn(work);
        t.cancel();
        t.cancel();
        let status = t.status();
        let done = t.is_done();
        "#,
    );

    result.expect("cancel doit être idempotent");
    assert_eq!(string(&vm, "status"), "cancelled");
    assert!(boolean(&vm, "done"));
}

#[test]
fn cancelled_channel_sender_is_removed_from_wait_queue() {
    let (vm, result) = run_script(
        r#"
        let c = channel(1);
        c.send(1);
        let results = [];

        func sender(ch) -> None {
            ch.send(2);
        }

        func scenario(ch, output) -> None {
            let t = spawn(sender, ch);
            yield();
            output.add(t.status());
            t.cancel();

            let cancelled = false;
            try {
                t.join();
            } catch (e: Err) {
                cancelled = e.kind == "TaskCancelled";
            }

            output.add(cancelled);
            output.add(ch.recv());
            output.add(ch.try_send(3));
            output.add(ch.recv());
        }

        let coordinator = spawn(scenario, c, results);
        coordinator.join();
        "#,
    );

    result.expect("l'annulation d'un sender bloqué doit nettoyer son attente");
    assert_eq!(array_len(&vm, "results"), 5);
    assert_eq!(string_at(&vm, "results", 0), "waiting");
    assert!(boolean_at(&vm, "results", 1));
    assert_eq!(integer_at(&vm, "results", 2), 1);
    assert!(boolean_at(&vm, "results", 3));
    assert_eq!(integer_at(&vm, "results", 4), 3);
}



#[test]
fn cancellation_runs_finally_and_releases_mutex() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let order = [];
        let owner_waiting = "";
        let waiter_waiting = "";
        let owner_cancelled = false;
        let locked = false;

        func owner(lock) -> None {
            lock.lock();
            order.add(1);
            try {
                sleep(1000);
            } finally {
                lock.unlock();
                order.add(2);
            }
        }

        func waiter(lock) -> None {
            lock.lock();
            order.add(3);
            lock.unlock();
        }

        func scenario() {
            let owner_task = spawn(owner, m);
            yield();
            owner_waiting = owner_task.status();

            let waiter_task = spawn(waiter, m);
            yield();
            waiter_waiting = waiter_task.status();

            owner_task.cancel();

            try {
                owner_task.join();
            } catch (e: Err) {
                owner_cancelled = e.kind == "TaskCancelled";
            }

            waiter_task.join();
            locked = m.is_locked();
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("finally doit libérer le mutex lors d'une annulation");
    assert_eq!(string(&vm, "owner_waiting"), "waiting");
    assert_eq!(string(&vm, "waiter_waiting"), "waiting");
    assert!(boolean(&vm, "owner_cancelled"));
    assert!(!boolean(&vm, "locked"));
    assert_eq!(array_len(&vm, "order"), 3);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 3);
}



// ============================================================================
// CHANNELS / SELECT
// ============================================================================

#[test]
fn unbounded_channel_is_fifo_and_reports_empty_state() {
    let (vm, result) = run_script(
        r#"
        let c = channel();
        c.send(10);
        c.send(20);
        let size_before = c.size();
        let a = c.recv();
        let b = c.recv();
        let empty = c.is_empty();
        let size_after = c.size();
        "#,
    );

    result.expect("le canal non borné doit conserver l'ordre FIFO");
    assert_eq!(integer(&vm, "size_before"), 2);
    assert_eq!(integer(&vm, "a"), 10);
    assert_eq!(integer(&vm, "b"), 20);
    assert!(boolean(&vm, "empty"));
    assert_eq!(integer(&vm, "size_after"), 0);
}

#[test]
fn channel_capacity_distinguishes_bounded_and_unbounded_channels() {
    let (vm, result) = run_script(
        r#"
        let bounded = channel(3);
        let unbounded = channel();
        let bounded_capacity = bounded.capacity().unwrap();
        let unbounded_capacity = unbounded.capacity();
        let unbounded_is_none = unbounded_capacity.is_none();
        "#,
    );

    result.expect("capacity doit distinguer les canaux bornés des non bornés");
    assert_eq!(integer(&vm, "bounded_capacity"), 3);
    assert!(boolean(&vm, "unbounded_is_none"));
}

#[test]
fn bounded_channel_rejects_try_send_when_full_and_accepts_after_recv() {
    let (vm, result) = run_script(
        r#"
        let c = channel(1);
        let first = c.try_send(1);
        let second = c.try_send(2);
        let full = c.is_full();
        let value = c.recv();
        let third = c.try_send(3);
        let value2 = c.recv();
        "#,
    );

    result.expect("le canal borné doit respecter sa capacité");
    assert!(boolean(&vm, "first"));
    assert!(!boolean(&vm, "second"));
    assert!(boolean(&vm, "full"));
    assert_eq!(integer(&vm, "value"), 1);
    assert!(boolean(&vm, "third"));
    assert_eq!(integer(&vm, "value2"), 3);
}

#[test]
fn blocked_receiver_is_woken_by_send() {
    let (vm, result) = run_script(
        r#"
        let c = channel();

        func receiver(ch) -> int {
            return ch.recv();
        }

        func sender(ch) -> None {
            ch.send(77);
        }

        let receiver_task = spawn(receiver, c);
        let sender_task = spawn(sender, c);
        let value = receiver_task.join();
        sender_task.join();
        "#,
    );

    result.expect("un receiver bloqué doit être réveillé par send");
    assert_eq!(integer(&vm, "value"), 77);
}



#[test]
fn blocked_senders_are_released_in_fifo_order() {
    let (vm, result) = run_script(
        r#"
        let c = channel(1);
        c.send(1);
        let received = [];
        let order = [];

        func sender(ch, id) -> None {
            ch.send(id);
            order.add(id);
        }

        func consumer(ch, output) -> None {
            yield();
            output.add(ch.recv());
            output.add(ch.recv());
            output.add(ch.recv());
        }

        let a = spawn(sender, c, 2);
        let b = spawn(sender, c, 3);
        let consumer_task = spawn(consumer, c, received);
        consumer_task.join();
        a.join();
        b.join();
        "#,
    );

    result.expect("les senders bloqués doivent respecter une attente FIFO");
    assert_eq!(array_len(&vm, "received"), 3);
    assert_eq!(integer_at(&vm, "received", 0), 1);
    assert_eq!(integer_at(&vm, "received", 1), 2);
    assert_eq!(integer_at(&vm, "received", 2), 3);
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 2);
    assert_eq!(integer_at(&vm, "order", 1), 3);
}



#[test]
fn blocked_receivers_are_released_in_fifo_order() {
    let (vm, result) = run_script(
        r#"
        let c = channel();
        let order = [];

        func receiver(ch, output) -> None {
            let value = ch.recv();
            output.add(value);
        }

        func sender(ch) -> None {
            yield();
            ch.send(10);
            ch.send(20);
        }

        let a = spawn(receiver, c, order);
        let b = spawn(receiver, c, order);
        let s = spawn(sender, c);
        s.join();
        a.join();
        b.join();
        "#,
    );

    result.expect("les receivers doivent respecter une attente FIFO");
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 10);
    assert_eq!(integer_at(&vm, "order", 1), 20);
}



#[test]
fn try_recv_returns_none_without_blocking_when_channel_is_empty() {
    let (vm, result) = run_script(
        r#"
        let c = channel();
        let value = c.try_recv();
        let is_none = value.is_none();
        "#,
    );

    result.expect("try_recv vide doit retourner None");
    assert!(boolean(&vm, "is_none"));
}

#[test]
fn close_preserves_buffered_values_then_marks_channel_closed() {
    let (vm, result) = run_script(
        r#"
        let c = channel(2);
        c.send(1);
        c.send(2);
        c.close();

        let closed = c.is_closed();
        let a = c.recv();
        let b = c.recv();
        let caught = false;
        let kind = "";

        try {
            c.recv();
        } catch (e: Err) {
            caught = true;
            kind = e.kind;
        }
        "#,
    );

    result.expect("la fermeture doit conserver les valeurs déjà bufferisées");
    assert!(boolean(&vm, "closed"));
    assert_eq!(integer(&vm, "a"), 1);
    assert_eq!(integer(&vm, "b"), 2);
    assert!(boolean(&vm, "caught"));
    assert_eq!(string(&vm, "kind"), "ChannelClosed");
}

#[test]
fn close_makes_try_send_fail_without_mutating_the_channel() {
    let (vm, result) = run_script(
        r#"
        let c = channel();
        c.close();
        let sent = c.try_send(1);
        let value = c.try_recv();
        let is_none = value.is_none();
        "#,
    );

    result.expect("try_send sur canal fermé doit échouer sans panic");
    assert!(!boolean(&vm, "sent"));
    assert!(boolean(&vm, "is_none"));
}

#[test]
fn blocked_receiver_is_woken_with_channel_closed_error() {
    let (vm, result) = run_script(
        r#"
        let c = channel();

        func receiver(ch) -> int {
            return ch.recv();
        }

        func closer(ch) -> None {
            ch.close();
        }

        let t = spawn(receiver, c);
        let closer_task = spawn(closer, c);
        let caught = false;
        let kind = "";
        try {
            t.join();
        } catch (e: Err) {
            caught = true;
            kind = e.kind;
        }
        closer_task.join();
        "#,
    );

    result.expect("la fermeture doit réveiller les receivers bloqués");
    assert!(boolean(&vm, "caught"));
    assert_eq!(string(&vm, "kind"), "ChannelClosed");
}



#[test]
fn select_receives_first_ready_case_and_reports_the_index() {
    let (vm, result) = run_script(
        r#"
        let a = channel();
        let b = channel();
        b.send(22);

        let result = select([a, b]);
        let index = result[0];
        let value = result[1];
        let closed = result[2];
        "#,
    );

    result.expect("select doit choisir le premier cas immédiatement prêt");
    assert_eq!(integer(&vm, "index"), 1);
    assert_eq!(integer(&vm, "value"), 22);
    assert!(!boolean(&vm, "closed"));
}

#[test]
fn select_waits_inside_a_task_and_resumes_with_the_selected_value() {
    let (vm, result) = run_script(
        r#"
        let a = channel();
        let b = channel();

        func worker(x, y) -> int {
            let result = select([x, y]);
            return result[1];
        }

        func sender(ch) -> None {
            ch.send(55);
        }

        let t = spawn(worker, a, b);
        let s = spawn(sender, b);
        let value = t.join();
        s.join();
        "#,
    );

    result.expect("select doit pouvoir suspendre une tâche");
    assert_eq!(integer(&vm, "value"), 55);
}



#[test]
fn select_timeout_zero_returns_the_timeout_tuple() {
    let (vm, result) = run_script(
        r#"
        let c = channel();
        let result = select([c], 0);
        let index = result[0];
        let value = result[1];
        let closed = result[2];
        "#,
    );

    result.expect("select avec timeout nul doit être déterministe");
    assert_eq!(integer(&vm, "index"), -1);
    assert!(matches!(global(&vm, "value"), Value::None));
    assert!(!boolean(&vm, "closed"));
}

#[test]
fn select_supports_send_cases() {
    let (vm, result) = run_script(
        r#"
        let c = channel(1);
        let result = select([(c, 99)]);
        let index = result[0];
        let closed = result[2];
        let value = c.recv();
        "#,
    );

    result.expect("select doit pouvoir choisir un cas d'envoi");
    assert_eq!(integer(&vm, "index"), 0);
    assert!(!boolean(&vm, "closed"));
    assert_eq!(integer(&vm, "value"), 99);
}

#[test]
fn select_reports_closed_receive_case() {
    let (vm, result) = run_script(
        r#"
        let c = channel();
        c.close();
        let result = select([c]);
        let index = result[0];
        let value = result[1];
        let closed = result[2];
        "#,
    );

    result.expect("select doit distinguer un canal fermé d'un timeout");
    assert_eq!(integer(&vm, "index"), 0);
    assert!(matches!(global(&vm, "value"), Value::None));
    assert!(boolean(&vm, "closed"));
}

// ============================================================================
// MUTEX
// ============================================================================

#[test]
fn mutex_reports_locked_state_and_try_lock_is_non_blocking() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let before = m.is_locked();
        m.lock();
        let during = m.is_locked();
        let second = m.try_lock();
        m.unlock();
        let after = m.is_locked();
        "#,
    );

    result.expect("mutex doit fonctionner depuis la VM principale");
    assert!(!boolean(&vm, "before"));
    assert!(boolean(&vm, "during"));
    assert!(!boolean(&vm, "second"));
    assert!(!boolean(&vm, "after"));
}

#[test]
fn mutex_waiters_are_released_in_fifo_order() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let order = [];

        func worker(lock, id) -> None {
            lock.lock();
            order.add(id);
            yield();
            lock.unlock();
        }

        let a = spawn(worker, m, 1);
        let b = spawn(worker, m, 2);
        let c = spawn(worker, m, 3);
        a.join();
        b.join();
        c.join();
        "#,
    );

    result.expect("les waiters du mutex doivent rester FIFO");
    assert_eq!(array_len(&vm, "order"), 3);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 3);
}



#[test]
fn mutex_is_released_automatically_when_owner_task_completes() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let order = [];

        func owner(lock, output) -> None {
            lock.lock();
            output.add(1);
            yield();
        }

        func waiter(lock, output) -> None {
            lock.lock();
            output.add(2);
            lock.unlock();
        }

        let owner_task = spawn(owner, m, order);
        let waiter_task = spawn(waiter, m, order);
        owner_task.join();
        waiter_task.join();
        let locked = m.is_locked();
        "#,
    );

    result.expect("un mutex doit être libéré lorsqu'une tâche se termine en le tenant");
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert!(!boolean(&vm, "locked"));
}



#[test]
fn reentrant_mutex_lock_reports_deadlock() {
    expect_runtime_error(
        r#"
        let m = mutex();

        func deadlock(lock) -> None {
            lock.lock();
            lock.lock();
        }

        let t = spawn(deadlock, m);
        t.join();
        "#,
        "MutexDeadlock",
    );
}

// ============================================================================
// RWLOCK
// ============================================================================

#[test]
fn rwlock_allows_multiple_readers_and_tracks_reader_count() {
    let (vm, result) = run_script(
        r#"
        let lock = rwlock();
        let order = [];
        let counts = [];

        func reader(rw, id, output, observed) -> None {
            rw.read_lock();
            output.add(id);
            observed.add(rw.reader_count());
            yield();
            output.add(id + 10);
            rw.read_unlock();
        }

        let a = spawn(reader, lock, 1, order, counts);
        let b = spawn(reader, lock, 2, order, counts);
        a.join();
        b.join();
        let read_locked = lock.is_read_locked();
        "#,
    );

    result.expect("un RwLock doit autoriser plusieurs lecteurs");
    assert_eq!(array_len(&vm, "counts"), 2);
    assert_eq!(integer_at(&vm, "counts", 0), 1);
    assert_eq!(integer_at(&vm, "counts", 1), 2);
    assert_eq!(array_len(&vm, "order"), 4);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 11);
    assert_eq!(integer_at(&vm, "order", 3), 12);
    assert!(!boolean(&vm, "read_locked"));
}



#[test]
fn rwlock_writer_waits_for_readers_and_is_woken_after_release() {
    let (vm, result) = run_script(
        r#"
        let lock = rwlock();
        let order = [];

        func reader(rw, output) -> None {
            rw.read_lock();
            output.add(1);
            yield();
            rw.read_unlock();
            output.add(2);
        }

        func writer(rw, output) -> None {
            rw.write_lock();
            output.add(3);
            rw.write_unlock();
        }

        let reader_task = spawn(reader, lock, order);
        let writer_task = spawn(writer, lock, order);
        writer_task.join();
        reader_task.join();
        let write_locked = lock.is_write_locked();
        "#,
    );

    result.expect("un écrivain doit attendre les lecteurs actifs");
    assert_eq!(array_len(&vm, "order"), 3);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 3);
    assert!(!boolean(&vm, "write_locked"));
}



#[test]
fn rwlock_try_methods_respect_current_ownership() {
    let (vm, result) = run_script(
        r#"
        let lock = rwlock();
        let results = [];

        func probe(rw, output) -> None {
            rw.read_lock();
            output.add(rw.try_write_lock());
            rw.read_unlock();
            output.add(rw.try_write_lock());
            output.add(rw.is_write_locked());
            rw.write_unlock();
        }

        let t = spawn(probe, lock, results);
        t.join();
        "#,
    );

    result.expect("les opérations try_* du RwLock doivent être cohérentes");
    assert_eq!(array_len(&vm, "results"), 3);
    assert!(!boolean_at(&vm, "results", 0));
    assert!(boolean_at(&vm, "results", 1));
    assert!(boolean_at(&vm, "results", 2));
}

#[test]
fn rwlock_try_read_lock_acquires_when_no_writer_is_present() {
    let (vm, result) = run_script(
        r#"
        let lock = rwlock();
        let result = [];

        func probe(rw, output) -> None {
            output.add(rw.try_read_lock());
            output.add(rw.reader_count());
            rw.read_unlock();
        }

        let t = spawn(probe, lock, result);
        t.join();
        "#,
    );

    result.expect("try_read_lock doit être non bloquant");
    assert_eq!(array_len(&vm, "result"), 2);
    assert!(boolean_at(&vm, "result", 0));
    assert_eq!(integer_at(&vm, "result", 1), 1);
}

#[test]
fn rwlock_is_released_automatically_when_owner_task_completes() {
    let (vm, result) = run_script(
        r#"
        let lock = rwlock();
        let order = [];

        func owner(rw, output) -> None {
            rw.read_lock();
            output.add(1);
            yield();
        }

        func writer(rw, output) -> None {
            rw.write_lock();
            output.add(2);
            rw.write_unlock();
        }

        let owner_task = spawn(owner, lock, order);
        let writer_task = spawn(writer, lock, order);
        owner_task.join();
        writer_task.join();
        let read_locked = lock.is_read_locked();
        let write_locked = lock.is_write_locked();
        "#,
    );

    result.expect("un RwLock doit être libéré à la terminaison du propriétaire");
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert!(!boolean(&vm, "read_locked"));
    assert!(!boolean(&vm, "write_locked"));
}



#[test]
fn reentrant_rwlock_write_reports_deadlock() {
    expect_runtime_error(
        r#"
        let lock = rwlock();

        func deadlock(rw) -> None {
            rw.write_lock();
            rw.write_lock();
        }

        let t = spawn(deadlock, lock);
        t.join();
        "#,
        "RwLockDeadlock",
    );
}

#[test]
fn rwlock_operations_require_a_task_context() {
    expect_runtime_error(
        r#"
        let lock = rwlock();
        lock.read_lock();
        "#,
        "TaskNotFound",
    );
}

// ============================================================================
// SEMAPHORE
// ============================================================================

#[test]
fn semaphore_exposes_capacity_and_available_permits() {
    let (vm, result) = run_script(
        r#"
        let s = semaphore(2);
        let capacity = s.capacity();
        let initial = s.available();
        "#,
    );

    result.expect("semaphore doit exposer sa capacité");
    assert_eq!(integer(&vm, "capacity"), 2);
    assert_eq!(integer(&vm, "initial"), 2);
}

#[test]
fn semaphore_try_acquire_and_release_update_available_permits() {
    let (vm, result) = run_script(
        r#"
        let s = semaphore(1);
        let results = [];

        func worker(sem, output) -> None {
            let first = sem.try_acquire();
            if first {
                output.add(1);
            } else {
                output.add(0);
            }
            output.add(sem.available());
            sem.release();
            output.add(sem.available());
        }

        let t = spawn(worker, s, results);
        t.join();
        let available = s.available();
        "#,
    );

    result.expect("un sémaphore doit rendre et reprendre ses permis");
    assert_eq!(integer(&vm, "available"), 1);
    assert_eq!(array_len(&vm, "results"), 3);
    assert_eq!(integer_at(&vm, "results", 0), 1);
    assert_eq!(integer_at(&vm, "results", 1), 0);
    assert_eq!(integer_at(&vm, "results", 2), 1);
}



#[test]
fn semaphore_waiters_are_released_in_fifo_order() {
    let (vm, result) = run_script(
        r#"
        let s = semaphore(1);
        let order = [];

        func worker(sem, id, output) -> None {
            sem.acquire();
            output.add(id);
            yield();
            sem.release();
        }

        let a = spawn(worker, s, 1, order);
        let b = spawn(worker, s, 2, order);
        let c = spawn(worker, s, 3, order);
        a.join();
        b.join();
        c.join();
        "#,
    );

    result.expect("les acquisitions bloquantes du sémaphore doivent rester FIFO");
    assert_eq!(array_len(&vm, "order"), 3);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 3);
}



#[test]
fn cancelling_a_semaphore_owner_releases_its_permit() {
    let (vm, result) = run_script(
        r#"
        let s = semaphore(1);
        let order = [];
        let owner_cancelled = false;
        let available = 0;

        func owner(sem, output) -> None {
            sem.acquire();
            output.add(1);
            sleep(1000);
            output.add(2);
        }

        func waiter(sem, output) -> None {
            sem.acquire();
            output.add(3);
            sem.release();
        }

        func scenario() {
            let owner_task = spawn(owner, s, order);
            yield();
            let waiter_task = spawn(waiter, s, order);
            yield();
            owner_task.cancel();

            try {
                owner_task.join();
            } catch (e: Err) {
                owner_cancelled = e.kind == "TaskCancelled";
            }

            waiter_task.join();
            available = s.available();
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("l'annulation doit libérer les permis détenus");
    assert!(boolean(&vm, "owner_cancelled"));
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 3);
    assert_eq!(integer(&vm, "available"), 1);
}



#[test]
fn semaphore_is_released_automatically_when_owner_task_completes() {
    let (vm, result) = run_script(
        r#"
        let s = semaphore(1);
        let order = [];

        func owner(sem, output) -> None {
            sem.acquire();
            output.add(1);
            yield();
        }

        func waiter(sem, output) -> None {
            sem.acquire();
            output.add(2);
            sem.release();
        }

        let owner_task = spawn(owner, s, order);
        let waiter_task = spawn(waiter, s, order);
        owner_task.join();
        waiter_task.join();
        let available = s.available();
        "#,
    );

    result.expect("un sémaphore doit être libéré à la terminaison du propriétaire");
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer(&vm, "available"), 1);
}



#[test]
fn semaphore_requires_a_positive_capacity() {
    expect_runtime_error(
        r#"
        let s = semaphore(0);
        "#,
        "SemaphoreNonPositive",
    );
}

#[test]
fn semaphore_operations_require_a_task_context() {
    expect_runtime_error(
        r#"
        let s = semaphore(1);
        s.acquire();
        "#,
        "TaskNotFound",
    );
}

// ============================================================================
// WAIT GROUP
// ============================================================================

#[test]
fn wait_group_blocks_until_all_workers_finish() {
    let (vm, result) = run_script(
        r#"
        let group = wait_group();
        let order = [];
        group.add(2);

        func waiter(g, output) -> None {
            g.wait();
            output.add(99);
        }

        func worker(g, output, id) -> None {
            output.add(id);
            g.done();
        }

        let w = spawn(waiter, group, order);
        let a = spawn(worker, group, order, 1);
        let b = spawn(worker, group, order, 2);

        w.join();
        a.join();
        b.join();
        let done = group.is_done();
        let count = group.count();
        "#,
    );

    result.expect("wait_group doit réveiller le waiter au dernier done");
    assert!(boolean(&vm, "done"));
    assert_eq!(integer(&vm, "count"), 0);
    assert_eq!(array_len(&vm, "order"), 3);
    assert_eq!(integer_at(&vm, "order", 0), 1);
    assert_eq!(integer_at(&vm, "order", 1), 2);
    assert_eq!(integer_at(&vm, "order", 2), 99);
}



#[test]
fn cancelled_wait_group_waiter_is_cleaned_up() {
    let (vm, result) = run_script(
        r#"
        let group = wait_group();
        group.add(1);
        let waiting = "";
        let cancelled = false;
        let done = false;

        func waiter(g) -> None {
            g.wait();
        }

        func scenario() {
            let t = spawn(waiter, group);
            yield();
            waiting = t.status();
            t.cancel();

            try {
                t.join();
            } catch (e: Err) {
                cancelled = e.kind == "TaskCancelled";
            }

            group.done();
            done = group.is_done();
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("l'annulation d'un waiter de wait_group doit retirer son attente");
    assert_eq!(string(&vm, "waiting"), "waiting");
    assert!(boolean(&vm, "cancelled"));
    assert!(boolean(&vm, "done"));
}



#[test]
fn wait_group_rejects_negative_and_underflow_operations() {
    let (vm, result) = run_script(
        r#"
        let group = wait_group();
        let negative = false;
        let underflow = false;

        try {
            group.add(-1);
        } catch (e: Err) {
            negative = e.kind == "WaitGroupNegativeCount";
        }

        try {
            group.done();
        } catch (e: Err) {
            underflow = e.kind == "WaitGroupUnderflow";
        }
        "#,
    );

    result.expect("wait_group doit refuser les compteurs incohérents");
    assert!(boolean(&vm, "negative"));
    assert!(boolean(&vm, "underflow"));
}

// ============================================================================
// BARRIER
// ============================================================================

#[test]
fn barrier_releases_a_generation_and_is_reusable() {
    let (vm, result) = run_script(
        r#"
        let barrier = barrier(2);
        let order = [];

        func worker(b, output, id) -> None {
            output.add(id);
            b.wait();
            output.add(id + 10);
            b.wait();
        }

        let a = spawn(worker, barrier, order, 1);
        let b = spawn(worker, barrier, order, 2);
        a.join();
        b.join();

        let generation = barrier.generation();
        let arrived = barrier.arrived();
        let broken = barrier.is_broken();
        let parties = barrier.parties();
        "#,
    );

    result.expect("barrier doit libérer et réutiliser ses générations");
    assert_eq!(integer(&vm, "generation"), 2);
    assert_eq!(integer(&vm, "arrived"), 0);
    assert!(!boolean(&vm, "broken"));
    assert_eq!(integer(&vm, "parties"), 2);
    assert_eq!(array_len(&vm, "order"), 4);
    assert_eq!(integer_at(&vm, "order", 0) + integer_at(&vm, "order", 1), 3);
    assert_eq!(integer_at(&vm, "order", 2) + integer_at(&vm, "order", 3), 23);
}



#[test]
fn cancelling_a_barrier_waiter_breaks_the_barrier() {
    let (vm, result) = run_script(
        r#"
        let barrier = barrier(2);
        let waiting = "";
        let cancelled = false;
        let broken = false;
        let other_broken = false;

        func waiter(b) -> None {
            b.wait();
        }

        func other(b) -> None {
            b.wait();
        }

        func scenario() {
            let t = spawn(waiter, barrier);
            yield();
            waiting = t.status();
            t.cancel();

            try {
                t.join();
            } catch (e: Err) {
                cancelled = e.kind == "TaskCancelled";
            }

            broken = barrier.is_broken();
            let other_task = spawn(other, barrier);
            try {
                other_task.join();
            } catch (e: Err) {
                other_broken = e.kind == "BarrierBroken";
            }
        }

        let coordinator = spawn(scenario);
        coordinator.join();
        "#,
    );

    result.expect("l'annulation d'un barrier waiter doit casser la génération");
    assert_eq!(string(&vm, "waiting"), "waiting");
    assert!(boolean(&vm, "cancelled"));
    assert!(boolean(&vm, "broken"));
    assert!(boolean(&vm, "other_broken"));
}



#[test]
fn barrier_requires_a_positive_party_count() {
    expect_runtime_error(
        r#"
        let b = barrier(0);
        "#,
        "BarrierNonPositive",
    );
}

// ============================================================================
// EVENT
// ============================================================================

#[test]
fn event_is_sticky_until_reset_and_wakes_all_waiters() {
    let (vm, result) = run_script(
        r#"
        let e = event();
        let order = [];

        func waiter(ev, output, id) -> None {
            ev.wait();
            output.add(id);
        }

        func setter(ev) -> None {
            yield();
            ev.set();
        }

        let a = spawn(waiter, e, order, 1);
        let b = spawn(waiter, e, order, 2);
        let s = spawn(setter, e);
        let before = e.is_set();
        s.join();
        a.join();
        b.join();
        let after_set = e.is_set();
        e.wait();
        e.reset();
        let after_reset = e.is_set();
        "#,
    );

    result.expect("event doit rester signalé jusqu'à reset");
    assert!(!boolean(&vm, "before"));
    assert!(boolean(&vm, "after_set"));
    assert!(!boolean(&vm, "after_reset"));
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0) + integer_at(&vm, "order", 1), 3);
}



// ============================================================================
// CONDVAR
// ============================================================================

#[test]
fn condvar_notify_one_reacquires_the_associated_mutex() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let cv = condvar(m);
        let order = [];
        let waiter_count_before = 0;

        func waiter(lock, condition, output) -> None {
            lock.lock();
            condition.wait();
            output.add(1);
            lock.unlock();
        }

        func signaler(lock, condition, output) -> None {
            lock.lock();
            output.add(2);
            waiter_count_before = condition.waiter_count();
            condition.notify_one();
            lock.unlock();
        }

        let w = spawn(waiter, m, cv, order);
        let s = spawn(signaler, m, cv, order);
        s.join();
        w.join();
        let waiter_count_after = cv.waiter_count();
        "#,
    );

    result.expect("notify_one doit réveiller un waiter puis lui rendre le mutex");
    assert_eq!(integer(&vm, "waiter_count_before"), 1);
    assert_eq!(integer(&vm, "waiter_count_after"), 0);
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0), 2);
    assert_eq!(integer_at(&vm, "order", 1), 1);
}



#[test]
fn condvar_notify_all_wakes_all_waiters() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let cv = condvar(m);
        let order = [];
        let waiter_count_before = 0;

        func waiter(lock, condition, output, id) -> None {
            lock.lock();
            condition.wait();
            output.add(id);
            lock.unlock();
        }

        func signal_all(lock, condition) -> None {
            lock.lock();
            waiter_count_before = condition.waiter_count();
            condition.notify_all();
            lock.unlock();
        }

        let a = spawn(waiter, m, cv, order, 1);
        let b = spawn(waiter, m, cv, order, 2);
        let s = spawn(signal_all, m, cv);
        s.join();
        a.join();
        b.join();
        "#,
    );

    result.expect("notify_all doit réveiller tous les waiters");
    assert_eq!(integer(&vm, "waiter_count_before"), 2);
    assert_eq!(array_len(&vm, "order"), 2);
    assert_eq!(integer_at(&vm, "order", 0) + integer_at(&vm, "order", 1), 3);
}



#[test]
fn condvar_wait_requires_the_associated_mutex_and_a_task() {
    expect_runtime_error(
        r#"
        let m = mutex();
        let cv = condvar(m);
        cv.wait();
        "#,
        "TaskNotFound",
    );
}

#[test]
fn condvar_rejects_a_non_mutex_constructor_argument() {
    expect_compile_error(
        r#"
        let cv = condvar(1);
        "#,
    );
}



// ============================================================================
// OWNERSHIP ERRORS
// ============================================================================

#[test]
fn mutex_unlock_by_a_non_owner_reports_an_error_without_corrupting_owner() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let kind = "";

        func bad(lock) -> None {
            lock.unlock();
        }

        func owner(lock) -> None {
            lock.lock();
            let bad_task = spawn(bad, lock);
            try {
                bad_task.join();
            } catch (e: Err) {
                kind = e.kind;
            }
            lock.unlock();
        }

        let owner_task = spawn(owner, m);
        owner_task.join();
        "#,
    );

    result.expect("un unlock par un non-propriétaire doit être géré comme une erreur de tâche");
    assert_eq!(string(&vm, "kind"), "MutexNotOwner");
}



#[test]
fn semaphore_release_by_a_non_owner_reports_an_error_without_consuming_the_permit() {
    let (vm, result) = run_script(
        r#"
        let s = semaphore(1);
        let kind = "";

        func bad(sem) -> None {
            sem.release();
        }

        func owner(sem) -> None {
            sem.acquire();
            let bad_task = spawn(bad, sem);
            try {
                bad_task.join();
            } catch (e: Err) {
                kind = e.kind;
            }
            sem.release();
        }

        let owner_task = spawn(owner, s);
        owner_task.join();
        let available = s.available();
        "#,
    );

    result.expect("un release par un non-propriétaire doit être refusé");
    assert_eq!(string(&vm, "kind"), "SemaphoreNotOwner");
    assert_eq!(integer(&vm, "available"), 1);
}



#[test]
fn rwlock_unlock_by_a_non_owner_reports_an_error_without_corrupting_owner() {
    let (vm, result) = run_script(
        r#"
        let lock = rwlock();
        let kind = "";

        func bad(rw) -> None {
            rw.write_unlock();
        }

        func owner(rw) -> None {
            rw.write_lock();
            let bad_task = spawn(bad, rw);
            try {
                bad_task.join();
            } catch (e: Err) {
                kind = e.kind;
            }
            rw.write_unlock();
        }

        let owner_task = spawn(owner, lock);
        owner_task.join();
        let locked = lock.is_write_locked();
        "#,
    );

    result.expect("un unlock RwLock par un non-propriétaire doit être refusé");
    assert_eq!(string(&vm, "kind"), "RwLockNotOwner");
    assert!(!boolean(&vm, "locked"));
}



#[test]
fn condvar_wait_by_a_non_owner_reports_an_error_without_releasing_the_mutex() {
    let (vm, result) = run_script(
        r#"
        let m = mutex();
        let cv = condvar(m);
        let kind = "";

        func bad(condition) -> None {
            condition.wait();
        }

        func owner(lock, condition) -> None {
            lock.lock();
            let bad_task = spawn(bad, condition);
            try {
                bad_task.join();
            } catch (e: Err) {
                kind = e.kind;
            }
            lock.unlock();
        }

        let owner_task = spawn(owner, m, cv);
        owner_task.join();
        "#,
    );

    result.expect("condvar.wait doit vérifier que la tâche possède le mutex associé");
    assert_eq!(string(&vm, "kind"), "CondvarNotOwner");
}



// ============================================================================
// API GUARDS
// ============================================================================

#[test]
fn yield_is_rejected_outside_a_task() {
    expect_runtime_error(
        r#"
        yield();
        "#,
        "YieldOutsideTask",
    );
}

#[test]
fn channel_requires_positive_capacity_when_bounded() {
    expect_runtime_error(
        r#"
        let c = channel(0);
        "#,
        "ChannelNonPositive",
    );
}

#[test]
fn async_function_result_type_is_task() {
    compile_only(
        r#"
        async func compute() -> int {
            return 1;
        }

        let task: Task<int> = compute();
        "#,
    );
}
