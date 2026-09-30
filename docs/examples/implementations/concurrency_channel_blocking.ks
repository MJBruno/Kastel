let ch = channel();
let log = [];

func consumer(name: str) -> int {
    log.add(name + ":waiting");
    let value = ch.recv();
    log.add(name + ":received=" + str(value));
    return value;
}

func producer() -> int {
    log.add("producer:send");
    ch.send(42);
    return 0;
}

let consumer_task = spawn(consumer, "consumer");
let producer_task = spawn(producer);

let value = consumer_task.join();
let producer_result = producer_task.join();

println(log);
println("value = " + str(value));
println("producer = " + str(producer_result));
println("channel empty = " + str(ch.is_empty()));
