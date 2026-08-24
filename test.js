
function make() {

    let x = 10;
    function get() {
        return x * 10;
    }

    return get;
}

let f = make();
print(f() * 2);