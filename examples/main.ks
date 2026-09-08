function make_counter() {
    let value = 42;

    function get_value() {
        return value;
    }

    return get_value;
}

let get = make_counter();

for i in range(500) {
    let temp = [i, i + 1, i + 2, i + 3];
}

println(get());