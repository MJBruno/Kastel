// Classe générique : champ T + constructeur + méthode retournant T.
class Box<T> {
    private let value: T;

    func initialize(value: T) {
        this.value = value;
    }

    func get() -> T {
        return this.value;
    }
}

let int_box: Box<int> = new Box(123);
let str_box: Box<str> = new Box<str>("hello");

println(int_box.get());
println(str_box.get());
