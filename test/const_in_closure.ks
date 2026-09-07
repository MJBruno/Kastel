println("TEST CONST + CLOSURE");

function make_reader() {
    const value = 42;

    function read() {
        return value;
    }

    return read;
}

let reader = make_reader();

println(reader());