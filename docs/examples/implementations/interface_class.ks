interface Addable: Add {
    func add(other: Self) -> Self;
}

class Money : Addable {
    let amount: int;

    func initialize(amount: int) { self.amount = amount; }

    func add(other: Self) -> Self {
        return new Money(self.amount + other.amount);
    }
}

func combine<T: Addable>(a: T, b: T) -> T {
    return a + b;
}

let total = combine<Money>(new Money(100), new Money(50));
println(total.amount);   // 150