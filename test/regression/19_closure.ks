func make_counter(start) {
    let value = start;
    return () => {
        value += 1;
        return value;
    };
}

let counter = make_counter(10);
println(counter());
println(counter());
println(counter());
