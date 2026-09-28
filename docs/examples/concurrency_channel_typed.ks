let ch: Channel<int> = channel();

ch.send(42);

let value: int = ch.recv();
let maybe: Option<int> = ch.try_recv();

println("value = " + str(value));
println("empty = " + str(maybe.is_none()));

let typed = channel<str>();
typed.send("hello");
println(typed.recv());
