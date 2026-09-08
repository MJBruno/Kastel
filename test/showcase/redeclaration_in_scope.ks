let x = 10;

if true {
    let x = 20; // interdit en Kastel
    println(x);
}

println(x);

// VALIDE

println("TEST SCOPE MODIFICATION");

let x = 10;

if true {
    x = 20;
    println(x);
}

println(x);