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

println("first = " + str(first_result));
println("second = " + str(second_result));
println("first_done = " + str(first_done));
println("second_status = " + second_status);