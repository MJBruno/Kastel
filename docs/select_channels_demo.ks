import std.thread
from std.thread import SelectResult

let a: Channel<int> = channel();
let b: Channel<int> = channel();
b.send(42);

let result: SelectResult<int> = thread.select_channels([a, b]);
println(result.index);
println(result.value.unwrap());
println(result.closed);

let timeout_result: SelectResult<int> = thread.select_channels([a], 0);
println(timeout_result.index);
println(timeout_result.value.is_none());
