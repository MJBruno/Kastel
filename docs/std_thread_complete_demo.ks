import std.thread

func compute(value: int) -> int {
    thread.yield();
    return value * 2;
}

func delayed(value: int) -> int {
    thread.sleep(10);
    return value + 1;
}

let first: Task<int> = thread.spawn(compute, 21);
let second: Task<int> = thread.spawn(delayed, 41);

// Give ready tasks an explicit scheduling opportunity.
thread.sleep(0);

let first_result = first.join();
let second_result = second.join();

let first_done = first.is_done();
let second_status = second.status();

println("first = " + first_result.to_string());
println("second = " + second_result.to_string());
println("first_done = " + first_done.to_string());
println("second_status = " + second_status);
