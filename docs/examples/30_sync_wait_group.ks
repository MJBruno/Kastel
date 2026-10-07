import std.sync.wait_group;
import std.thread;

let task = thread.spawn_task(func() {
    let group = wait_group.create();

    group.add(1);
    group.done();
    group.wait();

    return true;
});

let value = task.join();
println(value);

println("std.sync.wait_group: OK");
