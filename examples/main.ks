function create_graph() {
    let a = [];
    let b = {};

    function inner() {
        return a;
    }

    a.push(b);
    b.callback = inner;

    return inner;
}

let f = create_graph();

for i in range(1000) {
    let temp = [i, i + 1, i + 2];
}

println(f());