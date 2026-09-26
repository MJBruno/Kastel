// Types génériques imbriqués.
type Pair<A, B> = { first: A, second: B };

class Box<T> {
    private let value: T;

    func initialize(value: T) {
        this.value = value;
    }

    func get() -> T {
        return this.value;
    }
}

let box: Box<Pair<int, str>> = new Box({
    first: 42,
    second: "answer"
});

let pair: Pair<int, str> = box.get();

println(pair.first);
println(pair.second);
