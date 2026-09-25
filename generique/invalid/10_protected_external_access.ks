class Parent<T> {
    protected let value: T;

    func initialize(value: T) {
        this.value = value;
    }
}

let parent: Parent<int> = new Parent(10);
let value: int = parent.value;
