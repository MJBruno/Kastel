// Vérifie que f<T>(...) ne casse pas les comparaisons a < b.
func identity<T>(value: T) -> T {
    return value;
}

let a = 1;
let b = 2;
let smaller: bool = a < b;
let value: int = identity<int>(123);

println(smaller);
println(value);
