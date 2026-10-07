import std.sync.event;
import std.thread;

let task = thread.spawn_task(func() {
    let signal = event.create();

    println(signal.is_set());
    signal.set();
    println(signal.is_set());
    signal.reset();
    println(signal.is_set());

    return true;
});

let value = task.join();
println(value);

println("std.sync.event: OK");
