import std.sync.semaphore;
import std.thread;

let task = thread.spawn_task(func() {
    let semaphore = semaphore.create(2);

    println(semaphore.capacity());
    println(semaphore.available());

    semaphore.acquire();
    println(semaphore.available());

    semaphore.release();
    println(semaphore.available());

    return true;
});

let value = task.join();
println(value);

println("std.sync.semaphore: OK");
