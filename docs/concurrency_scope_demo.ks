import std.thread

let mutex = mutex();
let result = thread.with_lock(mutex, func() {
    return 40 + 2;
});

println(result);
println(mutex.is_locked());
