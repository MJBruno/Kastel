function make_counter() {
    let count = 0;

    function increment() {
        count += 1;
        return count;
    }

    return increment;
}

let counter = make_counter();

println(counter());
println(counter());
println(counter());