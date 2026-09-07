println("TEST BREAK BOUCLES IMBRIQUEES");

for i in range(3) {
    for j in range(5) {
        if j == 2 {
            break;
        }

        println(j);
    }
}