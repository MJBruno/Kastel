
for (x in [10, 20, 30]) {
    println(x);
}

println("===========================");
for (i in range(5)) {
    println(i);           // 0 1 2 3 4
}

println("===========================");
for (i in range(2, 10, 2)) {
    println(i);           // 2 4 6 8
}

println("===========================");
for (i in range(10, 0, -1)) {
    if (i == 5) { break; }
    println(i);            // 10 9 8 7 6
}

