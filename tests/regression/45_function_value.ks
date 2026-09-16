func twice(f, x) {
    return f(f(x));
}

let inc = x => x + 1;
println(twice(inc, 10));
