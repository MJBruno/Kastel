import std.sync.barrier;
import std.thread;

let task = thread.spawn_task(func() {
    let barrier = barrier.create(1);
    barrier.wait();
    return true;
});

let value = task.join();
println(value);

println("std.sync.barrier: OK");
