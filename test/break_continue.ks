println("TEST BREAK + CONTINUE");

for i in range(10) {
    if i == 2 {
        continue;
    }

    if i == 6 {
        break;
    }

    println(i);
}