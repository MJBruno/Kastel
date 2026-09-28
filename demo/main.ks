class Counter: Iterator<int> {
    let current: int;
    let stop: int;

    func initialize(stop: int) {
        self.current = 0;
        self.stop = stop;
    }

    func next() -> int {
        let value = self.current;
        self.current = self.current + 1;
        return value;
    }

    func has_next() -> bool {
        return self.current < self.stop;
    }
}

let iterator: Iterator<int> = new Counter(5);

let values = iterator
    .map(func(value) { return value * 2; })
    .filter(func(value) { return value >= 4; })
    .take(2)
    .collect();

println(values)