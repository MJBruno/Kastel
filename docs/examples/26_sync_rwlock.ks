import std.sync.rwlock;
import std.thread;

let task = thread.spawn_task(func() {
    let lock = rwlock.create();

    lock.read_lock();
    println(lock.reader_count());
    lock.read_unlock();

    lock.write_lock();
    println("write lock acquired");
    lock.write_unlock();

    return true;
});

let value = task.join();
println(value);

println("std.sync.rwlock: OK");
