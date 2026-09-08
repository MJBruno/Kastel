function consume() {
    let data = [10, 20, 30];

    for value in data {
        println(value);
    }
}

consume();

for i in range(1000) {
    let temp = [i, i + 1, i + 2];
}

println("iterator GC OK");