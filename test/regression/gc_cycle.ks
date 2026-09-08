function create_cycle() {
    let a = [];

    a.push(a);
}

create_cycle();

for i in range(1000) {
    let temp = [i, i + 1, i + 2];
}

println("GC OK");