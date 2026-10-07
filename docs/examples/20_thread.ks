import std.thread;

let task = thread.spawn_task(func() {
    thread.sleep_ms(10);
    println("worker termine");
    return 42;
});

let value = task.join();
println(value);

thread.yield_now();
println("std.thread: OK");
