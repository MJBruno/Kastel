import std.thread
from std.thread import SelectResult

let a = channel < int > ();
let b = channel < int > ();

b.send(42);

let result = thread.select_channels([a, b]);

println(result.index);
println(result.value);
println(result.closed);
