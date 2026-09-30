class Bag: Iterable<int> {
    func iter() -> Iterator<int> {
        return [1, 2, 3].iter();
    }
}

for value in new Bag() {
    println(value);
}