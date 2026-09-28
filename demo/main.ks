let log = [];

func worker() {
    let scratch = [1, 2, 3];
    log.add(scratch.size());
}

let kept = spawn(worker);
kept.join();

for i in range(0, 200) { 
    spawn(worker); 
}