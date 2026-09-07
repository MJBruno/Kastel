println("TEST PARAMETRE MUTABLE");

function increment(x) {
    x += 1;
    return x;
}

let value = 10;

println(increment(value));
println(value);