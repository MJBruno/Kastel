// Héritage générique + protected.
class Parent<T> {
    protected let value: T;

    func initialize(value: T) {
        this.value = value;
    }

    protected func get_value() -> T {
        return this.value;
    }
}

class Child: Parent<int> {
    func read() -> int {
        return this.value;
    }

    func read_through_method() -> int {
        return this.get_value();
    }
}

let child: Child = new Child(321);
let parent: Parent<int> = child;

println(child.read());
println(child.read_through_method());
