import std.sync.mutex;
import std.sync.condvar;
import std.thread;

let task = thread.spawn_task(func() {
    let lock = mutex.create();
    let condition = condvar.create(lock);

    lock.lock();
    condition.notify_one();
    condition.notify_all();
    lock.unlock();

    return true;
});

let value = task.join();
println(value);

println("std.sync.condvar: OK");
