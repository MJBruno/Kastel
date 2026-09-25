// Interface générique + implémentation.
interface Comparable<T> {
    func compare(other: T) -> int;
}

class Number: Comparable<int> {
    func compare(other: int) -> int {
        if other == 10 {
            return 0;
        }
        return 1;
    }
}

let value: Comparable<int> = new Number();
let result: int = value.compare(10);

println(result);
