function benchmark() {
    let i = 0;
    let sum = 0;

    while i < 1_000_000 {
        sum = sum + i;
        i = i + 1;
    }

    println(sum);
}

benchmark();