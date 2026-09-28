let channel = channel();

func producer() -> int {
    channel.send(10);
    channel.send(20);
    return 2;
}

let task = spawn(producer);
let produced = task.join();

println("produced = " + str(produced));
println("queue = " + str(channel.size()));

let first = channel.try_recv();
let second = channel.try_recv();
let missing = channel.try_recv();

println(first.to_string());
println(second.to_string());
println(missing.to_string());
println(channel.is_empty());