println("TEST TRUTHINESS");

if 0 {
    println("ERROR");
} else {
    println("0 = false");
}

if 1 {
    println("1 = true");
}

if "" {
    println("ERROR");
} else {
    println("empty = false");
}

if "Kastel" {
    println("string = true");
}

if null {
    println("ERROR");
} else {
    println("nil = false");
}