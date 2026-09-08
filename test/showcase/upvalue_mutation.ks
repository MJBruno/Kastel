println("TEST MUTATION UPVALUES");

function make_adder(a) {
    let b = 10;

    function add(x) {
        a += 1;
        b += 2;
        return a + b + x;
    }

    return add;
}

let add = make_adder(5);

println(add(1));
println(add(1));
println(add(1));