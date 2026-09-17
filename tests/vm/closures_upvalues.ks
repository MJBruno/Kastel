func make_counter() {
    let count = 0;
    func increment() {
        count = count + 1;
        return count;
    }
    return increment;
}

let counter1 = make_counter();
let counter2 = make_counter();

println(counter1());
println(counter1());
println(counter1());
println(counter2());

func make_adder(x) {
    func add(y) {
        return x + y;
    }
    return add;
}

let add5 = make_adder(5);
let add10 = make_adder(10);

println(add5(1));
println(add10(1));
