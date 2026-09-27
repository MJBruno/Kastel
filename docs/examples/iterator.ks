class Counter: Iterator<int> {
    let current: int;
    let stop: int;

    func initialize(stop: int) {
        this.current = 0;
        this.stop = stop;
    }

    func next() -> int {
        let value = this.current;
        this.current = this.current + 1;
        return value;
    }

    func has_next() -> bool {
        return this.current < this.stop;
    }
}

let iterator: Iterator<int> = new Counter(5);

let values = iterator
    .map(func(value) { return value * 2; })
    .filter(func(value) { return value >= 4; })
    .take(2)
    .collect();

println(values)