func add(a, b) {
    return a + b;
}

func fact(n) {
    if n <= 1 {
        return 1;
    }
    return n * fact(n - 1);
}

println(add(2, 3));
println(fact(6));
