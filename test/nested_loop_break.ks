println("TEST RETURN BOUCLES IMBRIQUEES");

function find_value() {
    for i in range(3) {
        for j in range(5) {
            if j == 2 {
                return i * 10 + j;
            }
        }
    }

    return -1;
}

println(find_value());