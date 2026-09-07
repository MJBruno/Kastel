println("TEST CLOSURES MULTIPLES");

function make_counter(start) {
    let count = start;

    function increment() {
        count += 1;
        return count;
    }

    return increment;
}

let counter_a = make_counter(0);
let counter_b = make_counter(100);

println(counter_a());
println(counter_a());
println(counter_b());
println(counter_b());
println(counter_a());