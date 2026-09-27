interface Add {
    func add(other: Money) -> int;
}

class Money : Add {
    let amount: int;

    func initialize(amount: int) {
        this.amount = amount;
    }

    func add(other: Money) -> int {
        return this.amount + other.amount;
    }

    
}


let a = new Money(100);
let b = new Money(50);

let total = a + b;

println(total)