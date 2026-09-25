interface Comparable<T> {
    func compare(other: T) -> int;
}

class Number: Comparable<int> {
    func compare(other: int) -> int {
        return 0;
    }
}

let invalid: Comparable<str> = new Number();
