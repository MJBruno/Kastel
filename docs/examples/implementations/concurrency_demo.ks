// Concurrence coopérative Kastel - V1
//
// spawn() crée une tâche.
// yield() rend explicitement la main au scheduler.
// Task.join() attend la fin et retourne T.
// Le scheduler préempte aussi automatiquement après son quantum.

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
