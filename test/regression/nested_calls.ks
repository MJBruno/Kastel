function add(a, b) {
    let value = a + b;
    return value;
}

function calculate(a, b) {
    let first = add(a, b);
    let second = add(first, 10);

    return second;
}

println(calculate(10, 20));