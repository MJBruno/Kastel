let first = channel<int>();
let second = channel<int>();

second.send(42);

let result = select([first, second]);




println(result[0])
println(result[1])
println(result[2])
// result[0] == 1
// result[1] == 42
// result[2] == false