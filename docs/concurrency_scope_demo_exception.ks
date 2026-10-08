import std.thread

let mutex = mutex();

try {
    thread.with_lock(mutex, func() {
        throw "boom";
    });
} catch(error) {
    println(error);
}

println(mutex.is_locked());
