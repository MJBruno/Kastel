println("TEST MULTIPLES PARAMETRES");

function calculate(a, b, c) {
    a += 1;
    b *= 2;
    c -= 3;

    return a + b + c;
}

println(calculate(10, 5, 8));