from std.collections import zip

let numbers: List<int> = [1, 2];
let names: List<str> = ["a", "b"];

let pairs: List<Tuple<int, str>> = zip(numbers, names);

for items in pairs {
    let x = items[0]
    let y = items[1]

    println("{0} : {1} ", x, y)
}
