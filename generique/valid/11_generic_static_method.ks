// Méthode statique générique.
class Utils {
    static func identity<T>(value: T) -> T {
        return value;
    }
}

let a: int = Utils.identity(99);
let b: str = Utils.identity<str>("static generic");

println(a);
println(b);
