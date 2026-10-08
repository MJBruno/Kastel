import std.thread

let task: Task<int> = thread.spawn_task(func() {
    return 42;
});

println(thread.task_status(task));
println(thread.task_done(task));
println(thread.join_task(task));
println(thread.task_done(task));
