println("TEST RETURN DANS BOUCLE");

function find_first() {
    for value in [10, 20, 30, 40] {
        if value == 30 {
            return value;
        }
    }

    return -1;
}

println(find_first());