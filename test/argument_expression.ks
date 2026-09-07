println("TEST ARGUMENTS EXPRESSIONS");

function multiply(a, b) {
    return a * b;
}

let x = 10;

println(multiply(x + 2, 3 * 4));
println(multiply(2 + 3, x - 5));