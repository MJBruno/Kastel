function make_function() {
    let value = [10, 20, 30];

    function get_value() {
        return value;
    }

    return get_value;
}

let get = make_function();

for i in range(1000) {
    let temp = [i, i + 1, i + 2];
}

println(get());