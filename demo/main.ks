func worker1() {
    println("worker1");
}
func worker2() {
    println("worker2");
}

let task1 = spawn(worker1);
let task2 = spawn(worker2);
task2.join();
task1.join();