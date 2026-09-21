// Classes - bound methods
class Counter {
    func initialize() {
        this.value = 0;
    }

    func increment() {
        this.value = this.value + 1;
        return this.value;
    }
}

let counter = new Counter();
let increment = counter.increment;

println(increment());
println(increment());
println(increment());
