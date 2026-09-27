class Box<T> {
    private let value: T;

    func initialize(value: T) {
        this.value = value;
    }

    func get() -> T {
        return this.value;
    }
}

let box: Box<int> = new Box(42);
let n: int = box.get();