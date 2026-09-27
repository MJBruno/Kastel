let log = [];

func worker(id: int) -> int {
    log.add(id);

    yield();

    log.add(id + 10);

    return id;
}

let first = spawn(worker, 1);
let second = spawn(worker, 2);

let a = first.join();
let b = second.join();

println(a);
println(b);
println(log);
println(first.status());
println(first.is_done());