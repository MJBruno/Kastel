import std.sync.channel;

let messages = channel.create<int>();

messages.send(10);
messages.send(20);

match messages.try_recv() {
    Some(value) => println(value);
    None => println("aucune valeur");
}

let value = messages.recv();
println(value);

messages.close();

println("std.sync.channel: OK");
