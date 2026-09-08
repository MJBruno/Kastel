println("TEST BREAK + CONTINUE IMBRIQUES");

for i in range(3) {
    for j in range(6) {
        if j == 1 {
            continue;
        }

        if j == 4 {
            break;
        }

        println(j);
    }
}