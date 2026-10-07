import std.ops;

func maximum<T: Ord>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    return b;
}

func add_values<T: Add>(a: T, b: T) -> T {
    return a + b;
}

println(maximum(3, 7));
println(add_values(10, 20));
println("std.ops: OK");
