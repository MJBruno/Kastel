 println("TEST RETURN CLOSURE");

function create_multiplier(factor) {
    function multiply(value) {
        return value * factor;
    }

    return multiply;
}

let double = create_multiplier(2);
let triple = create_multiplier(3);

println(double(5));
println(triple(5));
println(double(10));


// AVEC CONTROLFLOW

println("TEST RETURN CLOSURE");

function make_test() {
    let value = 10;

    function inner() {
        if value == 10 {
            return 42;
        }

        return 0;
    }

    return inner;
}

let test = make_test();

println(test());