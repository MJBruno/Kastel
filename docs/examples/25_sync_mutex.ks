import std.sync.mutex;
import std.thread;

let task = thread.spawn_task(func() {
    let lock = mutex.create();

    println(lock.is_locked());
    lock.lock();
    println(lock.is_locked());
    lock.unlock();
    println(lock.is_locked());

    return true;
});

let value = task.join();
println(value);

println("std.sync.mutex: OK");
