println("TEST ORDRE ARGUMENTS");

function test(a, b, c) {
    return a * 100 + b * 10 + c;
}

let x = 1;

println(test(x, x + 1, x + 2));