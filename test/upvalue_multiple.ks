println("TEST MULTIPLES UPVALUES");

function make_math(a) {
    let b = 10;

    function calculate(x) {
        return a + b + x;
    }

    return calculate;
}

let calc = make_math(5);

println(calc(1));
println(calc(10));