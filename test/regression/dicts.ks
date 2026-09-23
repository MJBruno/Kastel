// GC - dictionaries
let keep = {
    "name": "Bruno",
    "age": 1
};

for i in range(10000) {
    let temp = {
        index: i,
        value: "temporary"
    };
}

println(keep);
