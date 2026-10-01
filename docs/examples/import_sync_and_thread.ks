import std.thread
import std.sync.mutex
import std.sync.rwlock
import std.sync.semaphore
import std.sync.event
import std.sync.channel
import std.sync.condvar
import std.sync.wait_group
import std.sync.barrier

func mutex_worker(lock: Mutex, values: Channel<int>, group: WaitGroup) -> int {
    lock.lock();
    values.send(42);
    lock.unlock();
    group.done();
    return 7;
}

func event_worker(signal: Event, group: WaitGroup) -> int {
    signal.set();
    group.done();
    return 8;
}

func semaphore_worker(permits: Semaphore, group: WaitGroup) -> int {
    permits.acquire();
    permits.release();
    group.done();
    return 9;
}

func barrier_worker(rendezvous: Barrier) -> int {
    rendezvous.wait();
    return 10;
}

let mutex_lock = mutex.create();
let read_write = rwlock.create();
let permits = semaphore.create(2);
let ready = event.create();
let values = channel.create();
let bounded = channel.create_bounded(16);
let condition = condvar.create(mutex_lock);
let group = wait_group.create();
let rendezvous = barrier.create(1);

group.add(3);

let first_task = thread.spawn(mutex_worker, mutex_lock, values, group);
let second_task = thread.spawn(event_worker, ready, group);
let third_task = thread.spawn(semaphore_worker, permits, group);
let fourth_task = thread.spawn(barrier_worker, rendezvous);

let received = values.recv();
ready.wait();
group.wait();

let first_result = first_task.join();
let second_result = second_task.join();
let third_result = third_task.join();
let fourth_result = fourth_task.join();

condition.notify_all();

println("received = " + str(received));
println("first = " + str(first_result));
println("second = " + str(second_result));
println("third = " + str(third_result));
println("fourth = " + str(fourth_result));
println("event = " + str(ready.is_set()));
println("sem = " + str(permits.available()) + "/" + str(permits.capacity()));
println("channel = " + str(values.size()));
println("bounded_capacity = " + str(bounded.capacity()));
println("bounded_full = " + str(bounded.is_full()));
println("condvar_waiters = " + str(condition.waiter_count()));
println("group_done = " + str(group.is_done()));
println("rwlock = " + str(read_write.reader_count()) + "/" + str(read_write.is_write_locked()));
println("barrier = " + str(rendezvous.generation()));