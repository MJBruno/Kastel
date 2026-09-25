// Classe générique + méthode générique avec T et U dans le même contexte.
class Transformer<T> {
    private let value: T;

    func initialize(value: T) {
        this.value = value;
    }

    func get() -> T {
        return this.value;
    }

    func convert<U>(value: U) -> U {
        return value;
    }
}

let transformer: Transformer<int> = new Transformer(100);
let number: int = transformer.get();
let text: str = transformer.convert<str>("converted");

println(number);
println(text);
