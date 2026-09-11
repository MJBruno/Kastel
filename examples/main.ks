function benchmark() {
    let i = 0;

    while (i < 1000000) {
        i = i + 1;
    }

    return i;
}

println(benchmark());