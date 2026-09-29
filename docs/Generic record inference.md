
```Typescript

type Wrapper<U> = { value: U };

func unwrap<T>(box: Wrapper<T>) -> T {
    return box.value;
}

class RecordBox<T> {
    let value: T;

    func initialize(box: { value: T }) {
        self.value = box.value;
    }

    func get() -> T {
        return self.value;
    }
}

let boxed: RecordBox<int> = new RecordBox({ value: 7 });

println(boxed.get())
```
