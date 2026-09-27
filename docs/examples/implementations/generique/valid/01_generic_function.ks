// Fonction générique : inférence + argument explicite.
func identity<T>(value: T) -> T {
    return value;
}

let a: int = identity(42);
let b: str = identity<str>("Kastel");

println(a);
println(b);
