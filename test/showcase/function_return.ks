println("TEST FONCTIONS");

function add(a, b) {
    return a + b;
}

println(add(2, 3));
println(add(10, 20));

function test_return() {
    return 42;
    println("ERROR");
}

println(test_return());