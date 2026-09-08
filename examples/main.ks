function add(a, b) {
    return a + b;
}

function test() {
    let a = add(10, 20);
    let b = add(a, 30);
    return b;
}

println(test());