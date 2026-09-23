func make_accumulator() {
    let total = 0;
    func add(x) {
        total = total + x;
        return total;
    }
    return add;
}

let acc = make_accumulator();

let i = 0;
while i < 10000 {
    acc(1);
    let garbage = [i, i, i];
    i = i + 1;
}

println(acc(0));
